#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
//! Top-level crate of fydia
use fydia_config::get_config_or_init;

/// Start function
#[tokio::main]
async fn main() {
    let config = get_config_or_init();
    axum::Server::bind(&(config.format_ip().as_str()).parse().unwrap())
        .serve(
            fydia_router::get_axum_router(config)
                .await
                .into_make_service(),
        )
        .await
        .unwrap()
}
