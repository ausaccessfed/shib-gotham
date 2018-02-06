use std::fmt::Debug;
use std::panic::RefUnwindSafe;

use serde::Deserialize;

use gotham::router::Router;
use gotham::router::builder::*;

use receiver::{LoginHandler, Receiver, ReturnInfo};

/// Builds the subrouter for the Shibboleth-protected part of application, where new sessions will
/// be received for processing.
pub fn auth_router<A, R>(r: R) -> Router
where
    A: for<'de> Deserialize<'de> + Debug + 'static,
    R: Receiver<A> + Copy + RefUnwindSafe + 'static,
{
    build_simple_router(|route| {
        route
            .get("/login")
            .with_query_string_extractor::<ReturnInfo>()
            .to(LoginHandler::new(r));
    })
}
