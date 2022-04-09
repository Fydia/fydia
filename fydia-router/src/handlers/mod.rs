use axum::body::Bytes;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::StatusCode;
use serde_json::Value;

pub mod api;
pub mod basic;
pub mod event;
pub mod federation;

pub async fn default<'a>() -> FydiaResult<'a> {
    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "Default. This request will be implemented soon",
    ))
}

pub fn get_json_value_from_body(body: &Bytes) -> Result<Value, String> {
    let body = String::from_utf8(body.to_vec()).map_err(|_| "Bad Body".to_string())?;

    serde_json::from_str::<Value>(body.as_str()).map_err(|_| "Bad Body".to_string())
}

pub fn get_json<'a, T: Into<String>>(string: T, json: &Value) -> Result<&str, FydiaResponse<'a>> {
    let string = string.into();
    json.get(&string)
        .ok_or_else(|| FydiaResponse::StringError(format!("No `{string}` in JSON")))?
        .as_str()
        .ok_or_else(|| FydiaResponse::StringError(format!("`{string}` cannot be convert as str")))
}
