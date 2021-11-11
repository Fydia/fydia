use crate::handlers::api::instance::public_key::public_key;
use axum::response::IntoResponse;
use axum::Router;

/// All routes related to the instances
pub fn instance_routes() -> Router {
    Router::new()
        .route("/public_key", axum::routing::get(public_key))
        .route("/version", axum::routing::get(version))
}

/// Handler return version
pub async fn version() -> impl IntoResponse {
    String::from(env!("CARGO_PKG_VERSION"))
}
