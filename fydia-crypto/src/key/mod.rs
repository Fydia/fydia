use crate::{pem, PrivateKey, PublicKey};

pub mod generate;
pub mod io;

pub fn private_to_public(rsa: PrivateKey) -> PublicKey {
    let a = rsa.public_key_to_pem().unwrap();
    pem::get_key_from_string(String::from_utf8(a).unwrap()).unwrap()
}
