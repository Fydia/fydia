use crate::{pem, PrivateKey, PublicKey};

pub mod generate;
pub mod io;

pub fn private_to_public(rsa: PrivateKey) -> Option<PublicKey> {
    let pem_vector = rsa.public_key_to_pem().ok()?;
    let pem_string = String::from_utf8(pem_vector).ok()?;
    let public_key = pem::get_key_from_string(pem_string)?;

    Some(public_key)
}
