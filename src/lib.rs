//! Shibboleth SP authentication plugin for Gotham web applications

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate futures;
#[macro_use]
extern crate log;

mod middleware;
mod router;

pub use middleware::*;
pub use router::*;
