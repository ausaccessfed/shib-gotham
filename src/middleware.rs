use std::io;
use std::marker::PhantomData;

use futures::future;
use hyper::{StatusCode, Uri};
use hyper::header::Location;

use gotham::state::{State, FromState};
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::middleware::{NewMiddleware, Middleware};
use gotham::middleware::session::SessionData;

use authenticated_session::AuthenticatedSession;

trait SessionTypePhantom<T>: Send + Sync
where
    T: Send
{
}

/// Gotham middleware for receiving Shibboleth attributes and mapping them into a provided type.
pub struct Shibbleware<T>
where
    T: AuthenticatedSession,
{
    auth_login_location: &'static str,
    phantom: PhantomData<SessionTypePhantom<T>>,
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

impl<T> Copy for Shibbleware<T>
where
    T: AuthenticatedSession,
{
}

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

                response.headers_mut().set(Location::new(format!(
                    "{}?return_path={}",
                    self.auth_login_location,
                    return_path // TODO: URL encode
                )));
            }

            Box::new(future::ok((state, response)))
        }
    }
}
