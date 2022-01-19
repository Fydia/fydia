use axum::response::IntoResponse;
use fydia_struct::response::FydiaResponse;

use reqwest::StatusCode;

pub async fn delete_direct_message() -> impl IntoResponse {
    FydiaResponse::new_error_custom_status("", StatusCode::NOT_IMPLEMENTED)
}
