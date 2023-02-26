use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia};
use fydia_utils::serde_json::{self, Value};
pub mod api;
pub mod basic;
pub mod event;
pub mod federation;

/// Default response
///
/// # Errors
/// This function return an error by default
pub async fn default() -> FydiaResult {
    Err("Default. This request will be implemented soon".into_not_implemented_error())
}

/// Convert body to Json value
///
/// # Errors
/// This function will return an error if body cannot be convert to a json value
pub fn get_json_value_from_body(body: &String) -> Result<Value, FydiaResponse> {
    if body.is_empty() {
        return Err("Body is empty".into_error());
    }

    serde_json::from_str::<Value>(body.as_str()).map_err(|error| {
        error!("{error}");
        "Bad Body".into_error()
    })
}

/// Get a value from json
///
/// # Errors
/// This function will return an error if value isn't found
pub fn get_json<T: Into<String>>(string: T, json: &Value) -> Result<&str, FydiaResponse> {
    let string = string.into();
    json.get(&string)
        .ok_or_else(|| format!("No `{string}` in JSON").into_error())?
        .as_str()
        .ok_or_else(|| format!("`{string}` cannot be convert as str").into_error())
}
