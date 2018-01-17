use std::io;
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use futures::future;
use hyper::{Headers, Response, StatusCode};
use hyper::header::Location;
use serde::Deserialize;
use gotham::handler::{Handler, HandlerFuture, NewHandler};
use gotham::http::response::create_response;
use gotham::state::{request_id, FromState, State};

use headers::deserialize;

pub struct ReceiverFailed;

pub trait Receiver<A>: Send + Sync {
    fn receive(&self, &mut State, A) -> Result<(), ReceiverFailed>;

    fn finish(&self, &mut State) -> Response;
}

impl<A, F> Receiver<A> for F
where
    F: Fn(&mut State, A) -> Result<(), ReceiverFailed> + Send + Sync,
{
    fn receive(&self, state: &mut State, a: A) -> Result<(), ReceiverFailed> {
        self(state, a)
    }

    fn finish(&self, state: &mut State) -> Response {
        let return_info = ReturnInfo::take_from(state);

        let mut response = create_response(state, StatusCode::SeeOther, None);

        match return_info.return_path {
            Some(path) => response.headers_mut().set(Location::new(path)),
            None => response.headers_mut().set(Location::new("/")),
        }

        response
    }
}

trait AttributesTypePhantom<T>: Send + Sync + RefUnwindSafe
where
    T: Send,
{
}

#[derive(StateData, StaticResponseExtender, QueryStringExtractor)]
pub(crate) struct ReturnInfo {
    return_path: Option<String>,
}

pub struct LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
    r: R,
    phantom: PhantomData<AttributesTypePhantom<A>>,
}

impl<A, R> LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
    pub fn new(r: R) -> Self {
        LoginHandler {
            r,
            phantom: PhantomData,
        }
    }
}

impl<A, R> Copy for LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
}

impl<A, R> Clone for LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, R> NewHandler for LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
    type Instance = Self;

    fn new_handler(&self) -> Result<Self, io::Error> {
        Ok(self.clone())
    }
}

impl<A, R> Handler for LoginHandler<A, R>
where
    R: Receiver<A> + Send + Sync + Copy + RefUnwindSafe,
    A: for<'de> Deserialize<'de> + 'static,
{
    fn handle(self, mut state: State) -> Box<HandlerFuture> {
        let attrs = match deserialize::<A>(Headers::borrow_from(&state)) {
            Ok(t) => t,
            Err(e) => {
                error!(
                    "[{}] failed to deserialize user from incoming headers: {:?}",
                    request_id(&state),
                    e
                );

                let response = create_response(&state, StatusCode::InternalServerError, None);
                return Box::new(future::ok((state, response)));
            }
        };

        match self.r.receive(&mut state, attrs) {
            Ok(()) => {}
            Err(ReceiverFailed) => {
                let response = create_response(&state, StatusCode::InternalServerError, None);
                return Box::new(future::ok((state, response)));
            }
        }

        let response = self.r.finish(&mut state);
        Box::new(future::ok((state, response)))
    }
}
