//! This module is related to HTTP Response


use axum::{body, headers::HeaderName, response::IntoResponse};
use http::{HeaderValue, Response};
use hyper::{header::CONTENT_TYPE, HeaderMap, StatusCode};
use serde::Serialize;
use serde_json::Value;

/// FydiaResult is Result with a FydiaResult as Ok and Err
///
/// FydiaResult is used to return a response to Client
pub type FydiaResult = Result<FydiaResponse, FydiaResponse>;

/// FydiaStatus is used to know if something have failed
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub enum FydiaStatus {
    Ok,
    Error,
}

impl Default for FydiaStatus {
    fn default() -> Self {
        FydiaStatus::Error
    }
}

/// FydiaResponseBody used to know which type of data is in FydiaResponse
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum FydiaResponseBody {
    String(String),
    Json(Value),
    Bytes(Vec<u8>),
}

impl Default for FydiaResponseBody {
    fn default() -> Self {
        Self::String(String::from("Default value"))
    }
}

/// FydiaResponse is an easy struct to do HTTP response
#[allow(missing_docs)]
#[derive(Debug, Serialize, Default)]
pub struct FydiaResponse {
    status: FydiaStatus,
    #[serde(rename(serialize = "content"))]
    body: FydiaResponseBody,
    #[serde(skip)]
    statuscode: StatusCode,
    #[serde(skip)]
    headers: HeaderMap,
}

impl FydiaResponse {
    /// Take needed value and return a new `FydiaResponse`
    fn new(status: FydiaStatus, body: FydiaResponseBody, statuscode: StatusCode) -> Self {
        Self {
            status,
            body,
            statuscode,
            ..Default::default()
        }
    }
    
    /// Return a String if Ok or a error message as a String
    ///
    /// # Examples 
    /// ```
    /// use fydia_struct::response::FydiaResponse;
    ///
    /// let body = FydiaResponse::new_ok("EMPTYVALUE").get_body();
    ///
    /// assert!(body.is_ok())
    /// ```
    pub fn get_body(&self) -> Result<String, String> {
        match &self.body {
            FydiaResponseBody::String(string) => Ok(string.clone()),
            FydiaResponseBody::Json(_) => Err(String::from("Error body is not a string")),
            FydiaResponseBody::Bytes(_) => Err(String::from("Error body is not a string")),
        }
    }
    
    /// Create a new Ok `FydiaResponse` from a `Vec<u8>`
    pub fn new_bytes_ok(body: Vec<u8>) -> Self {
        Self::new(
            FydiaStatus::Ok,
            FydiaResponseBody::Bytes(body),
            StatusCode::OK,
        )
    }
    
    /// Create a new Error `FydiaResponse` from a `Vec<u8>`
    pub fn new_bytes_error(body: Vec<u8>) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::Bytes(body),
            StatusCode::BAD_REQUEST,
        )
    }

    /// Create a new Errror with custum status `FydiaResponse`  from a `Vec<u8>` 
    pub fn new_bytes_error_custom_status(body: Vec<u8>, status_code: StatusCode) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::Bytes(body),
            status_code,
        )
    }

    /// Create a new Error `FydiaResponse` from a `Into<Stirng>` value
    pub fn new_error<T: Into<String>>(body: T) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::String(body.into()),
            StatusCode::BAD_REQUEST,
        )
    }
    
    /// Create a new Error with a custom status `FydiaResponse` from a `Into<String>` value
    pub fn new_error_custom_status<T: Into<String>>(body: T, status_code: StatusCode) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::String(body.into()),
            status_code,
        )
    }
    
    /// Create a new Ok `FydiaResponse` from a `Into<String>` value 
    pub fn new_ok<T: Into<String>>(body: T) -> Self {
        Self::new(
            FydiaStatus::Ok,
            FydiaResponseBody::String(body.into()),
            StatusCode::OK,
        )
    }

    /// Create a new Ok `FydiaResponse` from a value that implement `Serialize` 
    pub fn new_ok_json<S: Serialize>(body: S) -> Self
    where
        S: Serialize,
    {
        match serde_json::to_string(&body) {
            Ok(body) => Self {
                status: FydiaStatus::Ok,
                body: FydiaResponseBody::Json(
                    serde_json::from_str::<Value>(&body).unwrap_or_default(),
                ),
                ..Default::default()
            },
            Err(e) => Self::new_error(format!(r#"{{"status":"Error", "content":{e}}}"#,)),
        }
    }
    
    /// Add header in `FydiaResponse` with name and value of header
    pub fn add_headers<T: Into<String>>(&mut self, name: T, value: T) -> Result<(), String> {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.into().as_bytes()),
            HeaderValue::from_bytes(value.into().as_bytes())
        ) {
            self.headers.insert(name, value);
            return Ok(());
        }

        Err("Cannot add header".to_string())
    }
}

impl IntoResponse for FydiaResponse {
    fn into_response(self) -> axum::response::Response {
        let mut response = Response::builder();
        let headers = response.headers_mut();

        match self.body {
            FydiaResponseBody::String(_) | FydiaResponseBody::Json(_) => {
                if let Some(headers) = headers {
                    if let Some(header) = headers.get_mut(CONTENT_TYPE) {
                        if header.to_str().unwrap_or_default() != mime::APPLICATION_JSON {
                            if let Ok(content_type) = mime::APPLICATION_JSON.to_string().parse() {
                                *header = content_type;
                            }
                        }
                    } else if let Ok(e) = mime::APPLICATION_JSON.to_string().parse() {
                        headers.insert(CONTENT_TYPE, e);
                    }
                }

                response = response.status(self.statuscode);

                match serde_json::to_string(&self) {
                    Ok(response_str) => response
                        .body(body::boxed(body::Full::from(response_str)))
                        .unwrap(),
                    Err(e) => response
                        .status(StatusCode::BAD_REQUEST)
                        .body(body::boxed(body::Full::from(format!(
                            r#"{{"status":"Error", "content":{e}}}"#,
                        ))))
                        .unwrap(),
                }
            }
            FydiaResponseBody::Bytes(body) => {
                response.body(body::boxed(body::Full::from(body))).unwrap()
            }
        }
    }
}
