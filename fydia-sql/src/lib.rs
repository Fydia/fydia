#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate log;

pub mod connection;
pub mod impls;

#[cfg(any(debug_assertions, feature = "sample"))]
pub mod samples;
pub mod setup;
pub mod sqlpool;
