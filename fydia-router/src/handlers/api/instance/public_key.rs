use crate::handlers::basic::Rsa;
use axum::response::IntoResponse;
use fydia_utils::http::StatusCode;

pub async fn public_key(Rsa(rsa): Rsa) -> impl IntoResponse {
    if let Some(pem) = fydia_crypto::pem::key_to_string(&rsa.1) {
        (StatusCode::OK, pem)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "".to_string())
    }
}
