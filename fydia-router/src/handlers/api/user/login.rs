use axum::body::Bytes;
use axum::extract::Extension;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::{
    response::{FydiaResponse, FydiaResult},
    user::User,
};
use http::StatusCode;
use serde_json::value;

pub async fn user_login(body: Bytes, Extension(database): Extension<DbConnection>) -> FydiaResult {
    let body = body.to_vec();
    if body.is_empty() {
        return Err(FydiaResponse::new_error("Bad Body"));
    }
    let body_string =
        String::from_utf8(body).map_err(|_| FydiaResponse::new_error("Body error"))?;

    let json = serde_json::from_str::<value::Value>(body_string.as_str())
        .map_err(|_| FydiaResponse::new_error("Json error"))?;

    let email = json
        .get("email")
        .ok_or_else(|| FydiaResponse::new_error("No `email` in JSON"))?
        .as_str()
        .ok_or_else(|| FydiaResponse::new_error("`email` cannot be convert as str"))?;

    let password = json
        .get("password")
        .ok_or_else(|| FydiaResponse::new_error("No `email` in JSON"))?
        .as_str()
        .ok_or_else(|| FydiaResponse::new_error("`email` cannot be convert as str"))?;

    let mut user = User::get_user_by_email_and_password(email, password, &database)
        .await
        .ok_or_else(|| FydiaResponse::new_error("User not exists"))?;

    user.update_token(&database).await.map_err(|_| {
        FydiaResponse::new_error_custom_status("Database Error", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    let token = user
        .token
        .ok_or_else(|| FydiaResponse::new_error("Token error"))?;

    Ok(FydiaResponse::new_ok(token))
}
