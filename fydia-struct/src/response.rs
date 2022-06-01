#![allow(clippy::unwrap_used)]

//! This module is related to HTTP Response

use axum::{body, response::IntoResponse};
use http::{header::CONTENT_TYPE, response::Builder, Response, StatusCode};
use mime::Mime;
use serde::Serialize;
use serde_json::{json, Value};

#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub enum FydiaStatus {
    Ok,
    Error,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    status: FydiaStatus,
    #[serde(rename(serialize = "content"))]
    body: Value,
}

/// `FydiaResult` type alias for Result with `FydiaResponse`
pub type FydiaResult<'a> = Result<FydiaResponse<'a>, FydiaResponse<'a>>;

/// `FydiaResponse` is the abstract struct to make a HTTP Response
#[allow(missing_docs)]
#[derive(Debug)]
pub enum FydiaResponse<'a> {
    Text(&'a str),
    TextError(&'a str),
    TextErrorWithStatusCode(StatusCode, &'a str),
    String(String),
    StringError(String),
    StringWithStatusCode(StatusCode, String),
    Json(Value),
    Bytes(Vec<u8>),
    BytesWithContentType(Vec<u8>, Mime),
}

impl<'a> FydiaResponse<'a> {
    /// Make a Ok HTTP Response from a Serializable value
    pub fn from_serialize<T: Serialize>(ser: T) -> Self {
        match serde_json::to_value(ser) {
            Ok(value) => FydiaResponse::Json(value),
            Err(_) => FydiaResponse::TextError("Cannot serialize"),
        }
    }
}

impl<'a> IntoResponse for FydiaResponse<'a> {
    fn into_response(self) -> axum::response::Response {
        let mut response = Response::builder();
        let build_response = |status: FydiaStatus, body: Value, response: Builder| {
            let mut response = match &status {
                FydiaStatus::Ok => response.status(StatusCode::OK),
                FydiaStatus::Error => response.status(StatusCode::BAD_REQUEST),
            };

            let res = ResponseFormat { status, body };
            let string = match serde_json::to_string(&res) {
                Ok(str) => str,
                Err(error) => {
                    println!("{}", error);
                    panic!("Cannot serialize response");
                }
            };

            if let Some(headers) = response.headers_mut() {
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

            let body = body::Full::from(string);
            response.body(body::boxed(body)).unwrap()
        };

        match self {
            FydiaResponse::Text(text) => {
                let value = json!(text);
                build_response(FydiaStatus::Ok, value, response)
            }
            FydiaResponse::TextError(text) => {
                let value = json!(text);
                build_response(FydiaStatus::Error, value, response)
            }
            FydiaResponse::TextErrorWithStatusCode(statuscode, text) => {
                let value = json!(text);
                let builder = response.status(statuscode);
                build_response(FydiaStatus::Ok, value, builder)
            }
            FydiaResponse::String(text) => {
                let value = json!(text);
                build_response(FydiaStatus::Ok, value, response)
            }
            FydiaResponse::StringError(text) => {
                let value = json!(text);
                build_response(FydiaStatus::Error, value, response)
            }
            FydiaResponse::StringWithStatusCode(statuscode, text) => {
                let value = json!(text);
                let builder = response.status(statuscode);
                build_response(FydiaStatus::Ok, value, builder)
            }
            FydiaResponse::Json(value) => build_response(FydiaStatus::Ok, value, response),
            FydiaResponse::Bytes(bytes) => {
                response.body(body::boxed(body::Full::from(bytes))).unwrap()
            }
            FydiaResponse::BytesWithContentType(bytes, contenttype) => {
                if let Some(headers) = response.headers_mut() {
                    if let Some(header) = headers.get_mut(CONTENT_TYPE) {
                        if let Ok(content_type) = contenttype.to_string().parse() {
                            *header = content_type;
                        }
                    }
                }

                response.body(body::boxed(body::Full::from(bytes))).unwrap()
            }
        }
    }
}

/// This trait can be use to compact `map` and `map_err`
pub trait FydiaMap<T, E> {
    /// The fydia map of the trait
    ///
    /// # Errors
    /// Return an error if the `Result` is an error
    fn fydia_map<'a, OF: FnOnce(T) -> FydiaResponse<'a>, EF: FnOnce(E) -> FydiaResponse<'a>>(
        self,
        ok_res: OF,
        err_res: EF,
    ) -> FydiaResult<'a>;
}

impl<T, E: std::fmt::Display> FydiaMap<T, E> for Result<T, E> {
    fn fydia_map<'a, 's, OF: FnOnce(T) -> FydiaResponse<'a>, EF: FnOnce(E) -> FydiaResponse<'a>>(
        self,
        ok_res: OF,
        err_res: EF,
    ) -> FydiaResult<'a> {
        match self {
            Ok(ok_value) => Ok(ok_res(ok_value)),
            Err(err_value) => {
                error!("{err_value}");
                Err(err_res(err_value))
            }
        }
    }
}
