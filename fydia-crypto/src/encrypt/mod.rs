use std::borrow::Cow;

use crate::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use crate::PublicKey;
use fydia_utils::generate_string;
use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::Cipher;

/// Encrypt with public key
///
/// # Errors
/// Return an error if :
/// * `T` value cannot be encrypted
pub fn encrypt<T: Into<String>>(rsa: &Rsa<Public>, string: T) -> Result<Vec<u8>, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.public_encrypt(string.into().as_bytes(), &mut buf, Padding::PKCS1) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e.to_string()),
    }
}

/// Encrypt with private key
///
/// # Errors
/// Return an error if :
/// * `T` value cannot be encrypted
pub fn private_encrypt<'a, T: Into<Cow<'a, str>>>(
    rsa: &Rsa<Private>,
    string: T,
) -> Result<Vec<u8>, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.private_encrypt(string.into().as_bytes(), &mut buf, Padding::PKCS1) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e.to_string()),
    }
}

/// Return encrypted AES key with RSA Public Key and Encrypted Body with aes key
/// If ok, the first Vec<u8> is encrypted key and the second is encrypted data
/// ```ignore
///                                 1                               3
/// 0                               6                               1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |             IV                |               KEY(16..48)       
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///              KEY(16..48)         |              BODY(48..n)     |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// # Errors
/// Return an error if :
/// * `T` value cannot be encrypted
pub fn aes_encrypt<T: Into<String>>(
    rsa: &PublicKey,
    string: T,
) -> Result<(Iv, AesKeyEncrypt, EncryptedBody), String> {
    let key = generate_string(32);
    let cipher = Cipher::aes_256_ctr();
    let iv = generate_string(16);
    let encrypted = openssl::symm::encrypt(
        cipher,
        key.as_bytes(),
        Some(iv.as_bytes()),
        string.into().as_bytes(),
    );

    match encrypt(rsa, key) {
        Ok(aes_key_encrypted) => match encrypted {
            Ok(data) => Ok((
                Iv(iv),
                AesKeyEncrypt(aes_key_encrypted),
                EncryptedBody(data),
            )),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e),
    }
}
