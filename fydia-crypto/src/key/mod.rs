use crate::{pem, PrivateKey, PublicKey};

pub mod generate;
pub mod io;

pub fn private_to_public(rsa: PrivateKey) -> Option<PublicKey> {
    if let Ok(pem_vector) = rsa.public_key_to_pem() {
        if let Ok(pem_string) = String::from_utf8(pem_vector)  {
            if let Some(public_key) = pem::get_key_from_string(pem_string) {
                return Some(public_key);
            }
        }
    }
    
    None
}
