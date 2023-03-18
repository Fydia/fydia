use crate::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use crate::PrivateKey;
use openssl::pkey::Public;
use openssl::symm::Cipher;
use openssl::{
    pkey::Private,
    rsa::{Padding, Rsa},
};

/// Decrypt body with RSA private key
///
/// # Errors
/// Return an error if :
/// * body cannot decrypted
/// * body cannot convert as String
pub fn private(rsa: &Rsa<Private>, string: &[u8]) -> Result<String, String> {
    let mut buf = vec![0; rsa.size() as usize];
    rsa.private_decrypt(string, &mut buf, Padding::PKCS1)
        .map_err(|f| f.to_string())?;

    String::from_utf8(buf).map_err(|error| error.to_string())
}

/// Decrypt body with RSA public key
///
/// # Errors
/// Return an error if :
/// * body cannot decrypted
/// * body cannot convert as String
pub fn public(rsa: &Rsa<Public>, string: &[u8]) -> Result<String, String> {
    let mut buf = vec![0; rsa.size() as usize];
    rsa.public_decrypt(string, &mut buf, Padding::PKCS1)
        .map_err(|f| f.to_string())?;

    String::from_utf8(buf).map_err(|error| error.to_string())
}

/// Decrypt body
///
/// # Errors
/// Return an error if :
/// * body cannot decrypted
/// * body cannot convert as String
pub fn aes(rsa: &PrivateKey, body: &(Iv, AesKeyEncrypt, EncryptedBody)) -> Result<String, String> {
    let decrypted = private(rsa, &body.1 .0)?;
    let aes_key = decrypted.split_at(32).0.to_string();

    let cipher = Cipher::aes_256_ctr();
    let body = openssl::symm::decrypt(
        cipher,
        aes_key.as_bytes(),
        Some(body.0 .0.as_bytes()),
        body.2 .0.as_slice(),
    )
    .map_err(|f| f.to_string())?;

    String::from_utf8(body).map_err(|f| f.to_string())
}
