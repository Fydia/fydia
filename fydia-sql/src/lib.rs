#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate gotham_derive;

#[macro_use]
extern crate anyhow;

pub mod impls;

pub mod connection;
pub mod default;
pub mod sqlpool;
