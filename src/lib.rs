//! Shibboleth SP authentication plugin for Gotham web applications

extern crate gotham;
extern crate hyper;
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
