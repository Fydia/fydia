use axum::{
    body::Body,
    extract::{BodyStream, Extension},
    response::IntoResponse,
};
use futures::StreamExt;
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{instance::Instance, response::FydiaResponse, user::User};

use axum::http::Request;
use serde_json::Value;

use crate::new_response;

pub async fn create_user(
    _request: Request<Body>,
    mut body: BodyStream,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    while let Some(Ok(body_bytes)) = body.next().await {
        let body = body_bytes.to_vec();
        if let Ok(e) = String::from_utf8(body) {
            if let Ok(json) = serde_json::from_str::<Value>(&e) {
                let name = json.get("name");
                let email = json.get("email");
                let password = json.get("password");

                match (name, email, password) {
                    (Some(name), Some(email), Some(password)) => {
                        match (name.as_str(), email.as_str(), password.as_str()) {
                            (Some(name), Some(email), Some(password)) => {
                                if let Err(e) =
                                    User::new(name, email, password, Instance::default())
                                        .insert_user(&database)
                                        .await
                                {
                                    error!(e);
                                    FydiaResponse::new_error("Database error")
                                        .update_response(&mut res);
                                } else {
                                    FydiaResponse::new_ok("Register successfully")
                                        .update_response(&mut res);
                                }
                            }
                            _ => {
                                FydiaResponse::new_error("Json error").update_response(&mut res);
                            }
                        }
                    }
                    _ => {
                        FydiaResponse::new_error("Json error").update_response(&mut res);
                    }
                }
            } else {
                FydiaResponse::new_error("Body is not json").update_response(&mut res);
            }
        } else {
            FydiaResponse::new_error("Utf-8 Body").update_response(&mut res);
        }
    }

    res
}
