use fydia_crypto::decrypt::public;
use fydia_crypto::structs::{AesKeyEncrypt, EncryptedBody, Iv};
use fydia_struct::instance::{Instance, RsaData};
use fydia_utils::http::HeaderMap;

pub async fn receive_message(headers: &HeaderMap, body: &[u8], rsa: &RsaData) -> Option<String> {
    let origin = headers.get("origin")?.to_str().ok()?.to_string();
    let authenticity = headers.get("Authenticity")?.to_str().ok()?.to_string();

    let public_key = crate::keys::get::get_public_key(Instance::from(origin)?).await?;

    let verification_bytes = &body[0..256];
    if public(&public_key, verification_bytes).ok()?.split_at(4).0 == authenticity {
        let iv = &body[256..272];
        let iv_string = String::from_utf8(iv.to_vec()).ok()?;
        let aeskey = &body[272..528];
        let body = &body[528..];
        let decrypted = fydia_crypto::decrypt::aes(
            &rsa.0,
            &(
                Iv(iv_string),
                AesKeyEncrypt(aeskey.to_vec()),
                EncryptedBody(body.to_vec()),
            ),
        )
        .ok()?;

        Some(decrypted)
    } else {
        None
    }
}
