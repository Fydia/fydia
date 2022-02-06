use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::StatusCode;
use serde_json::Value;

pub mod api;
pub mod basic;
pub mod event;
pub mod federation;

pub async fn default() -> FydiaResult {
    Err(FydiaResponse::new_error_custom_status(
        "Default. This request will be implemented soon",
        StatusCode::NOT_IMPLEMENTED,
    ))
}

pub fn get_json<T: Into<String>>(string: T, json: &Value) -> Result<&str, FydiaResponse> {
    let string = string.into();
    json.get(&string)
        .ok_or_else(|| FydiaResponse::new_error(format!("No `{}` in JSON", string)))?
        .as_str()
        .ok_or_else(|| FydiaResponse::new_error(format!("`{}` cannot be convert as str", string)))
}
