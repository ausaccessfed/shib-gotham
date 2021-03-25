use std::io;
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;

use futures::future;
use hyper::header::Location;
use hyper::{StatusCode, Uri};
use percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};

use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::middleware::session::SessionData;
use gotham::middleware::{Middleware, NewMiddleware};
use gotham::state::{FromState, State};

use authenticated_session::AuthenticatedSession;

trait SessionTypePhantom<T>: Send + Sync + RefUnwindSafe
where
    T: Send,
{
}

/// Gotham middleware for receiving Shibboleth attributes and mapping them into a provided type.
pub struct Shibbleware<T>
where
    T: AuthenticatedSession,
{
    auth_login_location: &'static str,
    phantom: PhantomData<dyn SessionTypePhantom<T>>,
}

impl<T> Shibbleware<T>
where
    T: AuthenticatedSession,
{
    pub fn new(auth_login_location: &'static str) -> Shibbleware<T> {
        Shibbleware {
            auth_login_location,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for Shibbleware<T> where T: AuthenticatedSession {}

impl<T> Clone for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> NewMiddleware for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    type Instance = Self;

    fn new_middleware(&self) -> io::Result<Self::Instance> {
        Ok(*self)
    }
}

define_encode_set! {
    pub QUERY_VALUE_ENCODE_SET = [QUERY_ENCODE_SET] | {'&', '=', ';'}
}

impl<T> Middleware for Shibbleware<T>
where
    T: AuthenticatedSession,
{
    fn call<Chain>(self, state: State, chain: Chain) -> Box<HandlerFuture>
    where
        Chain: FnOnce(State) -> Box<HandlerFuture>,
    {
        if SessionData::<T>::borrow_from(&state).is_authenticated() {
            chain(state)
        } else {
            let mut response = create_response(&state, StatusCode::SeeOther, None);

            {
                let uri = Uri::borrow_from(&state);

                let return_path = match uri.query() {
                    Some(query) => format!("{}?{}", uri.path(), query),
                    None => uri.path().to_owned(),
                };

                let encoded_return_path = utf8_percent_encode(&return_path, QUERY_VALUE_ENCODE_SET);

                response.headers_mut().set(Location::new(format!(
                    "{}?return_path={}",
                    self.auth_login_location, encoded_return_path
                )));
            }

            Box::new(future::ok((state, response)))
        }
    }
}
