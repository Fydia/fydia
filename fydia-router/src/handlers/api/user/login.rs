use crate::handlers::{get_json, get_json_value_from_body};
use axum::body::Bytes;
use axum::extract::Extension;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::{
    response::{FydiaResponse, FydiaResult},
    user::User,
};

/// Return a token
///
/// # Errors
/// This function return an error if body isn't valid or if user isn't exists
pub async fn user_login<'a>(
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let json = get_json_value_from_body(&body)?;

    let email = get_json("email", &json)?;
    let password = get_json("password", &json)?;

    let mut user = User::by_email_and_password(email, password, &database)
        .await
        .ok_or(FydiaResponse::TextError("User not exists"))?;

    user.update_token(&database).await?;

    let token = user.token.ok_or(FydiaResponse::Text("Token error"))?;

    Ok(FydiaResponse::String(token))
}
