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
    let body_string = if let Ok(value) = String::from_utf8(body) {
        value
    } else {
        return FydiaResponse::new_error("Body error");
    };
    let json = if let Ok(value) = serde_json::from_str::<value::Value>(body_string.as_str()) {
        value
    } else {
        return FydiaResponse::new_error("Json error");
    };

    match (json.get("email"), json.get("password")) {
        (Some(email), Some(password)) => match (email.as_str(), password.as_str()) {
            (Some(email), Some(password)) => {
                let user = User::get_user_by_email_and_password(email, password, &database).await;
                match user {
                    Some(mut user) => match user.update_token(&database).await {
                        Ok(_) => {
                            if let Some(token) = user.token {
                                FydiaResponse::new_ok(token)
                            } else {
                                FydiaResponse::new_error("Token error")
                            }
                        }
                        Err(error) => FydiaResponse::new_error(error),
                    },
                    None => FydiaResponse::new_error("User not exists"),
                }
            }
            _ => FydiaResponse::new_error("Json error"),
        },
        _ => FydiaResponse::new_error("Json error"),
    }
}
