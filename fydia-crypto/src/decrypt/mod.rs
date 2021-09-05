use crate::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use crate::PrivateKey;
use openssl::pkey::Public;
use openssl::symm::Cipher;
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

pub fn decrypt(rsa: &Rsa<Private>, string: Vec<u8>) -> Result<String, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.private_decrypt(string.as_slice(), &mut buf, Padding::PKCS1) {
        Ok(_) => match String::from_utf8(buf) {
            Ok(string) => Ok(string),
            Err(e) => Err(e.to_string()),
        },

        Err(e) => Err(e.to_string()),
    }
}

pub fn public_decrypt(rsa: &Rsa<Public>, string: Vec<u8>) -> Result<String, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.public_decrypt(string.as_slice(), &mut buf, Padding::PKCS1) {
        Ok(_) => match String::from_utf8(buf) {
            Ok(string) => Ok(string),
            Err(e) => Err(e.to_string()),
        },

        Err(e) => Err(e.to_string()),
    }
}

pub fn aes_decrypt(
    rsa: PrivateKey,
    body: (Iv, AesKeyEncrypt, EncryptedBody),
) -> Result<String, String> {
    let aes_key = match decrypt(&rsa, body.1 .0) {
        Ok(decrypted) => decrypted.split_at(32).0.to_string(),
        Err(e) => return Err(e),
    };

    let cipher = Cipher::aes_256_ctr();
    let try_body = openssl::symm::decrypt(
        cipher,
        aes_key.as_bytes(),
        Some(body.0 .0.as_bytes()),
        body.2 .0.as_slice(),
    );
    return match try_body {
        Ok(e) => Ok(String::from_utf8(e).unwrap_or_default()),
        Err(error) => {
            println!("{}", error);
            Err(error.to_string())
        }
    };
}
