//! Shibboleth SP authentication plugin for Gotham web applications

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

mod middleware;
mod router;
mod headers;

pub use middleware::*;
pub use router::*;
