use fydia_crypto::decrypt::public_decrypt;
use fydia_crypto::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use fydia_struct::instance::{Instance, RsaData};
use gotham::hyper::HeaderMap;

pub async fn receive_message(headers: &HeaderMap, body: Vec<u8>, rsa: &RsaData) -> Option<String> {
    let origin = headers.get("origin").unwrap().to_str().unwrap().to_string();
    let authenticity = headers
        .get("Authenticity")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let public_key = crate::keys::get::get_public_key(Instance::from(origin).unwrap())
        .await
        .unwrap();

    let verification_bytes = &body[0..256];
    if public_decrypt(&public_key, verification_bytes.to_vec())
        .unwrap()
        .split_at(4)
        .0
        == authenticity
    {
        let iv = &body[256..272];
        let iv_string = String::from_utf8(iv.to_vec()).unwrap();
        let aeskey = &body[272..528];
        let body = &body[528..];
        if let Ok(decrypted) = fydia_crypto::decrypt::aes_decrypt(
            rsa.0.clone(),
            (
                Iv(iv_string),
                AesKeyEncrypt(aeskey.to_vec()),
                EncryptedBody(body.to_vec()),
            ),
        ) {
            return Some(decrypted);
        }
    }

    None
}
