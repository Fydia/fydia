#![warn(missing_debug_implementations)]
#![deny(missing_docs)]
//! Top-level crate of fydia
use fydia_config::get_config_or_init;

/// Start function
#[tokio::main]
async fn main() {
    let config = get_config_or_init();

    gotham::init_server(config.format_ip(), fydia_router::get_router(config).await)
        .await
        .unwrap()
}
