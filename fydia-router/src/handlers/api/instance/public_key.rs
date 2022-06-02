use std::sync::Arc;

use axum::{extract::Extension, response::IntoResponse};
use fydia_struct::instance::RsaData;
use fydia_utils::http::StatusCode;

pub async fn public_key(Extension(rsa): Extension<Arc<RsaData>>) -> impl IntoResponse {
    if let Some(pem) = fydia_crypto::pem::key_to_string(&rsa.1) {
        (StatusCode::OK, pem)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
    }
}
