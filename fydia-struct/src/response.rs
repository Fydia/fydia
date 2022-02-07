#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use axum::{body, headers::HeaderName, response::IntoResponse};
use http::{HeaderValue, Response};
use hyper::{header::CONTENT_TYPE, HeaderMap, StatusCode};
use serde::Serialize;
use serde_json::Value;

pub type FydiaResult = Result<FydiaResponse, FydiaResponse>;

#[derive(Serialize)]
pub enum FydiaStatus {
    OK,
    Error,
}

impl Default for FydiaStatus {
    fn default() -> Self {
        FydiaStatus::Error
    }
}

#[derive(Serialize)]
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

#[derive(Serialize, Default)]
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
    fn new(status: FydiaStatus, body: FydiaResponseBody, statuscode: StatusCode) -> Self {
        Self {
            status,
            body,
            statuscode,
            ..Default::default()
        }
    }

    pub fn get_body(&self) -> Result<String, String> {
        match &self.body {
            FydiaResponseBody::String(string) => Ok(string.clone()),
            FydiaResponseBody::Json(_) => Err(String::from("Error body is not a string")),
            FydiaResponseBody::Bytes(_) => Err(String::from("Error body is not a string")),
        }
    }

    pub fn new_bytes_ok(body: Vec<u8>) -> Self {
        Self::new(
            FydiaStatus::OK,
            FydiaResponseBody::Bytes(body),
            StatusCode::OK,
        )
    }

    pub fn new_bytes_error(body: Vec<u8>) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::Bytes(body),
            StatusCode::BAD_REQUEST,
        )
    }

    pub fn new_bytes_error_custom_status(body: Vec<u8>, status_code: StatusCode) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::Bytes(body),
            status_code,
        )
    }

    pub fn new_error<T: Into<String>>(body: T) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::String(body.into()),
            StatusCode::BAD_REQUEST,
        )
    }

    pub fn new_error_custom_status<T: Into<String>>(body: T, status_code: StatusCode) -> Self {
        Self::new(
            FydiaStatus::Error,
            FydiaResponseBody::String(body.into()),
            status_code,
        )
    }

    pub fn new_ok<T: Into<String>>(body: T) -> Self {
        Self::new(
            FydiaStatus::OK,
            FydiaResponseBody::String(body.into()),
            StatusCode::OK,
        )
    }
    pub fn new_ok_json<S: Serialize>(body: S) -> Self
    where
        S: Serialize,
    {
        match serde_json::to_string(&body) {
            Ok(body) => Self {
                status: FydiaStatus::OK,
                body: FydiaResponseBody::Json(
                    serde_json::from_str::<Value>(&body).unwrap_or_default(),
                ),
                ..Default::default()
            },
            Err(e) => Self::new_error(format!(r#"{{"status":"Error", "content":{e}}}"#,)),
        }
    }

    pub fn add_headers<T: Into<String>>(&mut self, name: T, value: T) {
        self.headers.insert(
            HeaderName::from_bytes(name.into().as_bytes()).unwrap(),
            HeaderValue::from_bytes(value.into().as_bytes()).unwrap(),
        );
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
