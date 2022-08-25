use axum::body::Bytes;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::StatusCode;
use fydia_utils::serde_json::{self, Value};

pub mod api;
pub mod basic;
pub mod event;
pub mod federation;

/// Default response
///
/// # Errors
/// This function return an error by default
pub async fn default<'a>() -> FydiaResult<'a> {
    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "Default. This request will be implemented soon",
    ))
}

/// Convert body to Json value
///
/// # Errors
/// This function will return an error if body cannot be convert to a json value
pub fn get_json_value_from_body<'a>(body: &Bytes) -> Result<Value, FydiaResponse<'a>> {
    if body.is_empty() {
        return Err(FydiaResponse::TextError("Body is empty"));
    }

    let body = String::from_utf8(body.to_vec()).map_err(|error| {
        error!("{error}");
        FydiaResponse::TextError("Bad Body")
    })?;

    serde_json::from_str::<Value>(body.as_str()).map_err(|error| {
        error!("{error}");
        FydiaResponse::TextError("Bad Body")
    })
}

/// Get a value from json
///
/// # Errors
/// This function will return an error if value isn't found
pub fn get_json<'a, T: Into<String>>(string: T, json: &Value) -> Result<&str, FydiaResponse<'a>> {
    let string = string.into();
    json.get(&string)
        .ok_or_else(|| FydiaResponse::StringError(format!("No `{string}` in JSON")))?
        .as_str()
        .ok_or_else(|| FydiaResponse::StringError(format!("`{string}` cannot be convert as str")))
}
