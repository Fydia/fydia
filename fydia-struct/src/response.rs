use http::Response;
use hyper::{header::CONTENT_TYPE, Body, StatusCode};
use serde::Serialize;
use serde_json::Value;
#[derive(Serialize)]
pub struct FydiaResponse {
    status: FydiaStatus,
    #[serde(rename(serialize = "content"))]
    body: FydiaResponseBody,
    #[serde(skip)]
    custom_statuscode: Option<StatusCode>,
}
impl FydiaResponse {
    pub fn new<T: Into<String>>(status: FydiaStatus, body: T) -> Self {
        let body: String = body.into();
        Self {
            status,
            body: FydiaResponseBody::String(body),
            custom_statuscode: None,
        }
    }
    pub fn new_error<T: Into<String>>(body: T) -> Self {
        Self::new(FydiaStatus::Error, body)
    }

    pub fn new_error_custom_status<T: Into<String>>(body: T, status_code: StatusCode) -> Self {
        let mut s = Self::new(FydiaStatus::Error, body);
        s.custom_statuscode = Some(status_code);
        s
    }

    pub fn new_ok<T: Into<String>>(body: T) -> Self {
        Self::new(FydiaStatus::OK, body)
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
                custom_statuscode: None,
            },
            Err(e) => Self::new_error(format!(r#"{{"status":"Error", "content":{}}}"#, e)),
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
                if let Some(status_code) = self.custom_statuscode {
                    *res.status_mut() = status_code;
                } else {
                    if res.status() != StatusCode::BAD_REQUEST {
                        *res.status_mut() = StatusCode::BAD_REQUEST;
                    }
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
#[derive(Serialize)]
#[serde(untagged)]
pub enum FydiaResponseBody {
    String(String),
    Json(Value),
}
