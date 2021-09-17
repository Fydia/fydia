#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate gotham_derive;

pub mod channel;
pub mod emoji;
pub mod error;
pub mod event;
pub mod instance;
pub mod messages;
pub mod pathextractor;
pub mod permission;
pub mod querystring;
pub mod roles;
pub mod server;
pub mod user;
