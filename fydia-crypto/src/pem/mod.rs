use openssl::pkey::Public;
use openssl::rsa::Rsa;

pub fn get_key_from_string<T: Into<String>>(pem: T) -> Option<Rsa<Public>> {
    Rsa::public_key_from_pem(pem.into().as_bytes()).ok()
}

#[must_use]
pub fn key_to_string(key: &Rsa<Public>) -> Option<String> {
    let pem = key.public_key_to_pem().ok()?;
    String::from_utf8(pem).ok()
}
