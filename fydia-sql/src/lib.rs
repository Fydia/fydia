#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate log;

pub mod connection;
pub(crate) mod entity;
pub mod impls;

#[cfg(debug_assertions)]
pub mod samples;
pub mod setup;
pub mod sqlpool;
