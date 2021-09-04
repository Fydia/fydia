use openssl::pkey::Public;
use openssl::rsa::Rsa;

pub fn get_key_from_string(pem: String) -> Option<Rsa<Public>> {
    if let Ok(public_key) = Rsa::public_key_from_pem(pem.as_bytes()) {
        Some(public_key)
    } else {
        None
    }
}

pub fn key_to_string(key: &Rsa<Public>) -> Option<String> {
    if let Ok(pem) = key.public_key_to_pem() {
        if let Ok(string) = String::from_utf8(pem) {
            Some(string)
        } else {
            None
        }
    } else {
        None
    }
}
