#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
//! Top-level crate of fydia
use fydia_config::get_config_or_init;

/// Start function
#[tokio::main]
async fn main() {
    let mut config = get_config_or_init();
    if config.instance.domain.is_empty() {
        if let Ok(req) = reqwest::Client::new()
            .get("http://ifconfig.io")
            .header("User-Agent", "curl/7.55.1")
            .send()
            .await
        {
            if let Ok(text) = req.text().await {
                config.instance.domain = text;
            } else {
                panic!("Domain is not valid")
            }
        } else {
            panic!("Domain is not valid")
        }
    };
    axum::Server::bind(&(config.format_ip().as_str()).parse().unwrap())
        .serve(
            fydia_router::get_axum_router_from_config(config)
                .await
                .into_make_service(),
        )
        .await
        .unwrap()
}
