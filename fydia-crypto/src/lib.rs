#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use openssl::pkey::{Private, Public};
use openssl::rsa::Rsa;

pub mod decrypt;
pub mod encrypt;
pub mod key;
pub mod pem;
pub mod structs;

pub type PublicKey = Rsa<Public>;
pub type PrivateKey = Rsa<Private>;
