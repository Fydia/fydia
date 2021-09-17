use http::Response;
use hyper::{header::CONTENT_TYPE, Body, StatusCode};
use serde::Serialize;
#[derive(Serialize)]
pub struct FydiaResponse {
    status: FydiaStatus,
    #[serde(rename(serialize = "content"))]
    body: String,
}

impl FydiaResponse {
    pub fn new<T: Into<String>>(status: FydiaStatus, body: T) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    pub fn new_error<T: Into<String>>(body: T) -> Self {
        Self {
            status: FydiaStatus::Error,
            body: body.into(),
        }
    }

    pub fn new_ok<T: Into<String>>(body: T) -> Self {
        Self {
            status: FydiaStatus::OK,
            body: body.into(),
        }
    }

    pub fn update_response(&self, res: &mut Response<Body>) {
        if let Some(header) = res.headers_mut().get_mut(CONTENT_TYPE) {
            if header.to_str().unwrap_or_default() != mime::APPLICATION_JSON {
                if let Ok(content_type) = mime::APPLICATION_JSON.to_string().parse() {
                    *header = content_type;
                }
            }
        }
        match self.status {
            FydiaStatus::Error => {
                if res.status() != StatusCode::BAD_REQUEST {
                    *res.status_mut() = StatusCode::BAD_REQUEST;
                }
            }
            FydiaStatus::OK => {
                if res.status() != StatusCode::OK {
                    *res.status_mut() = StatusCode::OK;
                }
            }
        }

        match serde_json::to_string(self) {
            Ok(e) => *res.body_mut() = e.into(),
            Err(e) => {
                if res.status() != StatusCode::BAD_REQUEST {
                    *res.status_mut() = StatusCode::BAD_REQUEST;
                }
                *res.body_mut() = format!(r#"{{"status":"Error", "content":{}}}"#, e).into();
            }
        }
    }
}

#[derive(Serialize)]
pub enum FydiaStatus {
    OK,
    Error,
}
