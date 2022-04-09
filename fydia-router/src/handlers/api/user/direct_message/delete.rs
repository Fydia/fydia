use fydia_struct::response::{FydiaResponse, FydiaResult};

use reqwest::StatusCode;

pub async fn delete_direct_message<'a>() -> FydiaResult<'a> {
    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "",
    ))
}
