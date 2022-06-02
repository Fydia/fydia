use fydia_struct::response::{FydiaResponse, FydiaResult};

use fydia_utils::http::StatusCode;

/// Delete a dm
///
/// # Errors
/// This function will return an error if dm doesn't exist
pub async fn delete_direct_message<'a>() -> FydiaResult<'a> {
    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "",
    ))
}
