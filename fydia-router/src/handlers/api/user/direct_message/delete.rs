use fydia_struct::response::{FydiaResponse, FydiaResult};

use reqwest::StatusCode;

pub async fn delete_direct_message() -> FydiaResult {
    Err(FydiaResponse::new_error_custom_status(
        "",
        StatusCode::NOT_IMPLEMENTED,
    ))
}
