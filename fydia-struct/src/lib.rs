//! Fydia-struct represent major part of struct used by router or database traits.
//! User, channel, server struct is in this crates

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[macro_use]
extern crate log;

pub mod channel;
pub mod directmessage;
pub mod emoji;
pub mod event;
pub mod file;
pub mod format;
pub mod instance;
pub mod manager;
pub mod messages;
pub mod pathextractor;
pub mod permission;
pub mod querystring;
pub mod response;
pub mod roles;
pub mod server;
pub mod user;
pub mod utils;

#[cfg(test)]
mod test;
