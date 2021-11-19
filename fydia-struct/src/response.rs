use hyper::{header::CONTENT_TYPE, HeaderMap, StatusCode};
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

    pub fn update_response(&self, res: &mut (StatusCode, HeaderMap, String)) {
        if let Some(header) = res.1.get_mut(CONTENT_TYPE) {
            if header.to_str().unwrap_or_default() != mime::APPLICATION_JSON {
                if let Ok(content_type) = mime::APPLICATION_JSON.to_string().parse() {
                    *header = content_type;
                }
            }
        } else if let Ok(e) = mime::APPLICATION_JSON.to_string().parse() {
            res.1.insert(CONTENT_TYPE, e);
        }
        match self.status {
            FydiaStatus::Error => {
                if let Some(status_code) = self.custom_statuscode {
                    res.0 = status_code;
                } else if res.0 != StatusCode::BAD_REQUEST {
                    res.0 = StatusCode::BAD_REQUEST;
                }
            }
            FydiaStatus::OK => {
                if res.0 != StatusCode::OK {
                    res.0 = StatusCode::OK;
                }
            }
        }
        match serde_json::to_string(self) {
            Ok(response) => res.2 = response,
            Err(e) => {
                if res.0 != StatusCode::BAD_REQUEST {
                    res.0 = StatusCode::BAD_REQUEST;
                }
                res.2 = format!(r#"{{"status":"Error", "content":{}}}"#, e);
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
