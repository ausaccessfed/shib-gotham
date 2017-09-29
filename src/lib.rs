//! Shibboleth SP authentication plugin for Gotham web applications

extern crate futures;
extern crate hyper;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate serde;
#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate serde_bytes;

mod middleware;
mod router;
mod headers;
mod receiver;

pub use middleware::*;
pub use router::*;
pub use receiver::*;
