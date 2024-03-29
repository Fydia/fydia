use fydia_crypto::encrypt::private;
use fydia_crypto::PublicKey;
use fydia_struct::event::Event;
use fydia_struct::instance::{Instance, RsaData};
use fydia_utils::generate_string;
use fydia_utils::serde_json;

/// # Panics
/// Panic if json serialization failed
pub fn encrypt_message(rsa_origin: &RsaData, key: &PublicKey, message: &Event) -> Vec<u8> {
    let json = serde_json::to_string(&message).unwrap();
    let (iv, aeskey, body) = fydia_crypto::encrypt::aes(key, json).unwrap();
    let string = generate_string(4);
    let verification = private(&rsa_origin.0, string).unwrap();
    let mut vec: Vec<u8> = Vec::new();

    vec.extend_from_slice(verification.as_slice());
    vec.extend_from_slice(iv.0.as_bytes());
    vec.extend_from_slice(aeskey.0.as_slice());
    vec.extend_from_slice(body.0.as_slice());

    vec
}

/// Send a message
///
/// # Errors
/// Return an error if:
/// * message isn't serializable
/// * aes encrypt isn't possible
/// * rsa encrypt isn't possible
pub fn send_message(
    rsa_origin: &RsaData,
    origin: &Instance,
    key: &PublicKey,
    message: &Event,
    instances: &[Instance],
) -> Result<(), String> {
    let json = serde_json::to_string(message).map_err(|error| error.to_string())?;
    let (iv, aeskey, body) = fydia_crypto::encrypt::aes(key, json)?;
    let string = generate_string(4);
    let verification = private(&rsa_origin.0, string.as_str())?;
    let mut vec: Vec<u8> = Vec::new();

    vec.extend_from_slice(verification.as_slice());
    vec.extend_from_slice(iv.0.as_bytes());
    vec.extend_from_slice(aeskey.0.as_slice());
    vec.extend_from_slice(body.0.as_slice());

    for i in instances.iter() {
        let (origin, i, vec, string) = (origin.clone(), i.clone(), vec.clone(), string.clone());
        tokio::spawn(send_message_to_instance(origin, i, vec, string));
    }

    Ok(())
}

async fn send_message_to_instance(
    origin: Instance,
    to: Instance,
    vec: Vec<u8>,
    authenticity: String,
) {
    reqwest::Client::new()
        .post(format!("{}/api/federation/event/send", &to.format()))
        .header("Origin", origin.format())
        .header("Authenticity", authenticity)
        .body(vec.clone())
        .send()
        .await
        .expect("Error");
}
