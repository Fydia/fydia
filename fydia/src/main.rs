#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
//! Top-level crate of fydia
use fydia_config::get_config_or_init;

/// Start function
#[tokio::main]
async fn main() -> Result<(), ()> {
    let mut config = get_config_or_init();
    if config.instance.domain.is_empty() {
        let req = reqwest::Client::new()
            .get("http://ifconfig.io")
            .header("User-Agent", "curl/7.55.1")
            .send()
            .await
            .map_err(|_| {
                panic!("Domain is not valid");
            })?;

        let text = req
            .text()
            .await
            .map_err(|_| panic!("Domain is not valid"))?;

        config.instance.domain = text;
    };

    axum::Server::bind(&(config.format_ip().as_str()).parse().unwrap())
        .serve(
            fydia_router::get_axum_router_from_config(config)
                .await
                .into_make_service(),
        )
        .await
        .unwrap();

    Ok(())
}
