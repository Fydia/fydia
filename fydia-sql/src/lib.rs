#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate gotham_derive;

pub mod impls;

pub mod connection;
pub mod default;
pub mod sqlpool;
