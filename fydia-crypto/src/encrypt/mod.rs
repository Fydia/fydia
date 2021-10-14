use crate::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use crate::PublicKey;
use fydia_utils::generate_string;
use openssl::pkey::{Private, Public};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::Cipher;

pub fn encrypt(rsa: Rsa<Public>, string: String) -> Result<Vec<u8>, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.public_encrypt(string.as_bytes(), &mut buf, Padding::PKCS1) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e.to_string()),
    }
}

pub fn private_encrypt(rsa: Rsa<Private>, string: String) -> Result<Vec<u8>, String> {
    let mut buf = vec![0; rsa.size() as usize];

    match rsa.private_encrypt(string.as_bytes(), &mut buf, Padding::PKCS1) {
        Ok(_) => Ok(buf),
        Err(e) => Err(e.to_string()),
    }
}

/// Return encrypted AES key with RSA Public Key and Encrypted Body with aes key
/// If ok, the first Vec<u8> is encrypted key and the second is encrypted data
/// ```
///                                 1                               3
/// 0                               6                               1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |             IV                |               KEY(16..48)       
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///              KEY(16..48)         |              BODY(48..n)     |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
pub fn aes_encrypt(
    rsa: PublicKey,
    string: String,
) -> Result<(Iv, AesKeyEncrypt, EncryptedBody), String> {
    let key = generate_string(32);
    let cipher = Cipher::aes_256_ctr();
    let iv = generate_string(16);
    let encrypted = openssl::symm::encrypt(
        cipher,
        key.as_bytes(),
        Some(iv.as_bytes()),
        string.as_bytes(),
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
