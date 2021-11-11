use axum::response::IntoResponse;
use fydia_struct::response::FydiaResponse;

use reqwest::StatusCode;

use crate::new_response;

pub async fn delete_direct_message() -> impl IntoResponse {
    let mut res = new_response();

    FydiaResponse::new_error_custom_status("", StatusCode::NOT_IMPLEMENTED)
        .update_response(&mut res);
    res
}
