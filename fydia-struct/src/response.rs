#![allow(clippy::unwrap_used)]

//! This module is related to HTTP Response

use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::FromResidual;

use axum::{body, response::IntoResponse};
use fydia_utils::http::{header::CONTENT_TYPE, response::Builder, Response, StatusCode};
use fydia_utils::{
    serde::Serialize,
    serde_json::{self, json, Value},
};
use mime::Mime;

#[allow(missing_docs)]
#[derive(Debug, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub enum FydiaStatus {
    Ok,
    Error,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct ResponseFormat {
    status: FydiaStatus,
    #[serde(rename(serialize = "content"))]
    body: Value,
}

#[derive(Debug)]
/// Result for `FydiaResponse`
pub enum FydiaResult {
    /// Ok value
    Ok(FydiaResponse),
    /// Error value
    Err(FydiaResponse),
}

impl<E: Display> FromResidual<Result<Infallible, E>> for FydiaResult {
    #[inline]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(error) => Self::Err(FydiaResponse::StringError(Box::new(error.to_string()))),
            Ok(_) => panic!("wut cannot go here"),
        }
    }
}

impl FromResidual<Result<Infallible, FydiaResponse>> for FydiaResult {
    #[inline]
    fn from_residual(residual: Result<Infallible, FydiaResponse>) -> Self {
        match residual {
            Err(error) => Self::Err(error),
            Ok(_) => panic!("wut cannot go here"),
        }
    }
}

impl<T: IntoFydiaResponse> From<T> for FydiaResult {
    fn from(value: T) -> Self {
        FydiaResult::Ok(value.response_ok())
    }
}

impl From<FydiaResponse> for FydiaResult {
    fn from(value: FydiaResponse) -> Self {
        FydiaResult::Ok(value)
    }
}

impl From<Result<FydiaResponse, FydiaResponse>> for FydiaResult {
    fn from(value: Result<FydiaResponse, FydiaResponse>) -> Self {
        match value {
            Ok(displayok) => FydiaResult::Ok(displayok),
            Err(error) => FydiaResult::Err(error),
        }
    }
}

impl<T: IntoFydiaResponse, E: Display> From<Result<T, E>> for FydiaResult {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(displayok) => FydiaResult::Ok(displayok.response_ok()),
            Err(error) => {
                FydiaResult::Err(FydiaResponse::StringError(Box::new(format!("{error}"))))
            }
        }
    }
}

trait IntoFydiaResponse: Sized {
    fn response_ok(self) -> FydiaResponse;
    fn response_err(self) -> FydiaResponse;
}

impl IntoFydiaResponse for String {
    fn response_ok(self) -> FydiaResponse {
        FydiaResponse::String(Box::new(self))
    }

    fn response_err(self) -> FydiaResponse {
        FydiaResponse::StringError(Box::new(self))
    }
}

impl IntoFydiaResponse for &'static str {
    fn response_ok(self) -> FydiaResponse {
        FydiaResponse::Text(self)
    }

    fn response_err(self) -> FydiaResponse {
        FydiaResponse::Text(self)
    }
}

impl IntoResponse for FydiaResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            FydiaResult::Ok(response) => response.into_response(),
            FydiaResult::Err(err) => err.into_response(),
        }
    }
}

/// `ImplString` is used by `FydiaResponse` for generics type
pub trait ImplString: ToString + Debug + Send {}

impl ImplString for String {}
impl<'a> ImplString for &'a str {}

/// `FydiaResponse` is the abstract struct to make a HTTP Response
#[allow(missing_docs)]
#[derive(Debug)]
pub enum FydiaResponse {
    Text(&'static str),
    TextError(&'static str),
    TextErrorWithStatusCode(StatusCode, &'static str),
    String(Box<dyn ImplString>),
    StringError(Box<dyn ImplString>),
    StringWithStatusCode(StatusCode, Box<dyn ImplString>),
    Json(Value),
    Bytes(Vec<u8>),
    BytesWithContentType(Vec<u8>, Mime),
}

impl FydiaResponse {
    /// Make a Ok HTTP Response from a Serializable value
    pub fn from_serialize<T: Serialize>(ser: T) -> Self {
        match serde_json::to_value(ser) {
            Ok(value) => FydiaResponse::Json(value),
            Err(_) => "Cannot serialize".into_server_error(),
        }
    }

    /// Return a `String` from `FydiaResponse`
    pub fn get_string(self) -> String {
        match self {
            FydiaResponse::Text(str)
            | FydiaResponse::TextError(str)
            | FydiaResponse::TextErrorWithStatusCode(_, str) => str.to_string(),
            FydiaResponse::String(str)
            | FydiaResponse::StringError(str)
            | FydiaResponse::StringWithStatusCode(_, str) => str.to_string(),
            FydiaResponse::Json(_) => String::from("Json type cannot return a string"),
            FydiaResponse::Bytes(_) => String::from("Bytes type cannot return a string"),
            FydiaResponse::BytesWithContentType(_, _) => {
                String::from("BytesWithContentType type cannot return a string")
            }
        }
    }
}

impl IntoResponse for FydiaResponse {
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
                    println!("{error}");
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
            FydiaResponse::String(text) => {
                build_response(FydiaStatus::Ok, json!(text.to_string()), response)
            }
            FydiaResponse::StringError(text) => {
                build_response(FydiaStatus::Error, json!(text.to_string()), response)
            }
            FydiaResponse::StringWithStatusCode(statuscode, text) => build_response(
                FydiaStatus::Ok,
                json!(text.to_string()),
                response.status(statuscode),
            ),
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
            FydiaResponse::Text(str) => {
                build_response(FydiaStatus::Ok, json!(str.to_string()), response)
            }
            FydiaResponse::TextError(str) => {
                build_response(FydiaStatus::Error, json!(str.to_string()), response)
            }
            FydiaResponse::TextErrorWithStatusCode(statuscode, str) => build_response(
                FydiaStatus::Ok,
                json!(str.to_string()),
                response.status(statuscode),
            ),
        }
    }
}

impl<T: Error + Display> From<T> for FydiaResponse {
    fn from(value: T) -> Self {
        FydiaResponse::StringError(Box::new(value.to_string()))
    }
}

/// Convert types to `FydiaResponse`
pub trait IntoFydia {
    /// Convert this type to error with custom statuscode
    fn into_error_with_statuscode(self, statuscode: StatusCode) -> FydiaResponse
    where
        Self: Sized;

    /// Convert this type to `INTERNAL_SERVER_ERROR` error
    fn into_server_error(self) -> FydiaResponse
    where
        Self: Sized,
    {
        Self::into_error_with_statuscode(self, StatusCode::INTERNAL_SERVER_ERROR)
    }
    /// Convert this type to `FORBIDDEN` error
    fn into_forbidden_error(self) -> FydiaResponse
    where
        Self: Sized,
    {
        Self::into_error_with_statuscode(self, StatusCode::FORBIDDEN)
    }

    /// Convert this type to `NOT_IMPLEMENTED` error
    fn into_not_implemented_error(self) -> FydiaResponse
    where
        Self: Sized,
    {
        Self::into_error_with_statuscode(self, StatusCode::NOT_IMPLEMENTED)
    }
}

impl IntoFydia for &'static str {
    fn into_error_with_statuscode(self, statuscode: StatusCode) -> FydiaResponse {
        FydiaResponse::StringWithStatusCode(statuscode, Box::new(self))
    }
}

impl IntoFydia for String {
    fn into_error_with_statuscode(self, statuscode: StatusCode) -> FydiaResponse {
        FydiaResponse::StringWithStatusCode(statuscode, Box::new(self))
    }
}
