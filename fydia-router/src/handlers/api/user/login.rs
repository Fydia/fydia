use axum::body::Bytes;
use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::{response::FydiaResponse, user::User};
use serde_json::value;

pub async fn user_login(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let body = body.to_vec();
    if body.is_empty() {
        return FydiaResponse::new_error("Bad Body");
    }
    if let Ok(stringed_body) = String::from_utf8(body) {
        if let Ok(json) = serde_json::from_str::<value::Value>(stringed_body.as_str()) {
            match (json.get("email"), json.get("password")) {
                (Some(email), Some(password)) => match (email.as_str(), password.as_str()) {
                    (Some(email), Some(password)) => {
                        let user =
                            User::get_user_by_email_and_password(email, password, &database).await;

                        match user {
                            Some(mut user) => {
                                if let Ok(token) = user.update_token(&database).await {
                                    return FydiaResponse::new_ok(token);
                                } else {
                                    return FydiaResponse::new_error("Token error");
                                }
                            }
                            None => return FydiaResponse::new_error("User not exists"),
                        }
                    }
                    _ => return FydiaResponse::new_error("Json error"),
                },
                _ => return FydiaResponse::new_error("Json error"),
            }
        }
    }

    FydiaResponse::new_error("Body error")
}
