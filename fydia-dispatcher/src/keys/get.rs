use fydia_crypto::PublicKey;
use fydia_struct::instance::Instance;

pub async fn get_public_key(instance: Instance) -> Option<PublicKey> {
    let response = reqwest::get(format!(
        "http://{}:{}/api/instance/public_key",
        instance.domain, instance.port
    ))
    .await
    .unwrap();

    if let Ok(text) = response.text().await {
        if let Some(key) = fydia_crypto::pem::get_key_from_string(text) {
            return Some(key);
        }
    }

    None
}
