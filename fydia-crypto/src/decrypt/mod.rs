use crate::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use crate::PrivateKey;
use openssl::pkey::Public;
use openssl::symm::Cipher;
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

pub fn decrypt(rsa: &Rsa<Private>, string: Vec<u8>) -> Result<String, ()> {
    let mut buf = vec![0; rsa.size() as usize];

    if let Ok(_) = rsa.private_decrypt(string.as_slice(), &mut buf, Padding::PKCS1) {
        if let Ok(string) = String::from_utf8(buf) {
            Ok(string)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub fn public_decrypt(rsa: &Rsa<Public>, string: Vec<u8>) -> Result<String, ()> {
    let mut buf = vec![0; rsa.size() as usize];

    if let Ok(_) = rsa.public_decrypt(string.as_slice(), &mut buf, Padding::PKCS1) {
        if let Ok(string) = String::from_utf8(buf) {
            Ok(string)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub fn aes_decrypt(
    rsa: PrivateKey,
    body: (Iv, AesKeyEncrypt, EncryptedBody),
) -> Result<String, ()> {
    let aes_key = decrypt(&rsa, body.1 .0).unwrap().split_at(32).0.to_string();
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
            Err(())
        }
    };
}
