#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub mod channel;
pub mod emoji;
pub mod event;
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

#[cfg(test)]
mod test;
