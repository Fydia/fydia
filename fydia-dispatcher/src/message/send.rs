use fydia_crypto::encrypt::private_encrypt;
use fydia_crypto::PublicKey;
use fydia_struct::event::Event;
use fydia_struct::instance::{Instance, RsaData};
use fydia_utils::generate_string;
pub fn encrypt_message(rsa_origin: &RsaData, key: PublicKey, message: Event) -> Vec<u8> {
    let json = serde_json::to_string(&message).unwrap();
    let (iv, aeskey, body) = fydia_crypto::encrypt::aes_encrypt(key, json).unwrap();
    let string = generate_string(4);
    let verification = private_encrypt(rsa_origin.0.clone(), string).unwrap();
    let mut vec: Vec<u8> = Vec::new();

    vec.extend_from_slice(verification.as_slice());
    vec.extend_from_slice(iv.0.as_bytes());
    vec.extend_from_slice(aeskey.0.as_slice());
    vec.extend_from_slice(body.0.as_slice());

    vec
}
pub async fn send_message(
    rsa_origin: &RsaData,
    origin: Instance,
    key: PublicKey,
    message: Event,
    instances: Vec<Instance>,
) -> Result<(), ()> {
    let json = serde_json::to_string(&message).unwrap();
    let (iv, aeskey, body) = fydia_crypto::encrypt::aes_encrypt(key, json).unwrap();
    let string = generate_string(4);
    let verification = private_encrypt(rsa_origin.0.clone(), string.clone()).unwrap();
    let mut vec: Vec<u8> = Vec::new();

    vec.extend_from_slice(verification.as_slice());
    vec.extend_from_slice(iv.0.as_bytes());
    vec.extend_from_slice(aeskey.0.as_slice());
    vec.extend_from_slice(body.0.as_slice());

    for i in instances {
        tokio::spawn(send_message_to_instance(
            origin.clone(),
            i.clone(),
            vec.clone(),
            string.clone(),
        ));
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
        .body(vec)
        .send()
        .await
        .expect("Error");
}
