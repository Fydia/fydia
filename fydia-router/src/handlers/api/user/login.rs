use axum::extract::{BodyStream, Extension};
use axum::response::IntoResponse;
use futures::StreamExt;
use fydia_sql::sqlpool::DbConnection;
use serde_json::value;

use fydia_sql::impls::user::SqlUser;
use fydia_struct::{response::FydiaResponse, user::User};

use crate::new_response;

pub async fn user_login(
    mut body: BodyStream,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();

    if let Some(Ok(body_bytes)) = body.next().await {
        let body = body_bytes.to_vec();
        if body.is_empty() {
            FydiaResponse::new_error("Bad Body").update_response(&mut res);
            return res;
        }
        if let Ok(stringed_body) = String::from_utf8(body) {
            if let Ok(json) = serde_json::from_str::<value::Value>(stringed_body.as_str()) {
                match (json.get("email"), json.get("password")) {
                    (Some(email), Some(password)) => match (email.as_str(), password.as_str()) {
                        (Some(email), Some(password)) => {
                            let user = User::get_user_by_email_and_password(
                                email.to_string(),
                                password.to_string(),
                                &database,
                            )
                            .await;

                            match user {
                                Some(mut user) => {
                                    if let Ok(token) = user.update_token(&database).await {
                                        FydiaResponse::new_ok(token).update_response(&mut res);
                                    } else {
                                        FydiaResponse::new_error("Token error")
                                            .update_response(&mut res);
                                    }
                                }
                                None => {
                                    FydiaResponse::new_error("User not exists")
                                        .update_response(&mut res);
                                }
                            }
                        }
                        _ => {
                            FydiaResponse::new_error("Json error").update_response(&mut res);
                        }
                    },
                    _ => {
                        FydiaResponse::new_error("Json error").update_response(&mut res);
                    }
                }
            }
        }
    }

    res
}
