use fydia_crypto::PublicKey;
use fydia_struct::instance::Instance;

pub async fn get_public_key(instance: Instance) -> Option<PublicKey> {
    let response = reqwest::get(format!(
        "http://{}:{}/api/instance/public_key",
        instance.domain, instance.port
    ))
    .await
    .unwrap();
    let res = response.text().await.ok()?;
    Some(fydia_crypto::pem::get_key_from_string(res)?)
}
