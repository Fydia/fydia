#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate logger;

pub mod connection;
pub mod entity;
pub mod impls;

#[cfg(debug_assertions)]
pub mod samples;
pub mod setup;
pub mod sqlpool;
