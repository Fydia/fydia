use crate::handlers::get_json;
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

pub async fn user_login<'a>(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let body = body.to_vec();
    if body.is_empty() {
        return Err(FydiaResponse::TextError("Bad Body"));
    }
    let body_string =
        String::from_utf8(body).map_err(|_| FydiaResponse::TextError("Body error"))?;

    let json = serde_json::from_str::<value::Value>(body_string.as_str())
        .map_err(|_| FydiaResponse::TextError("Json error"))?;

    let email = get_json("email", &json)?;
    let password = get_json("password", &json)?;

    let mut user = User::get_user_by_email_and_password(email, password, &database)
        .await
        .ok_or(FydiaResponse::TextError("User not exists"))?;

    user.update_token(&database).await.map_err(|_| {
        FydiaResponse::TextErrorWithStatusCode(StatusCode::INTERNAL_SERVER_ERROR, "Database Error")
    })?;

    let token = user.token.ok_or(FydiaResponse::Text("Token error"))?;

    Ok(FydiaResponse::String(token))
}
