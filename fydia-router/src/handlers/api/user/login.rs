use crate::{
    handlers::{get_json, get_json_value_from_body},
    ServerState,
};

use axum::extract::State;
use fydia_sql::impls::user::SqlUser;
use fydia_struct::{
    response::{FydiaResult, IntoFydia},
    user::User,
};

/// Return a token
///
/// # Errors
/// This function return an error if body isn't valid or if user isn't exists
pub async fn user_login(State(state): State<ServerState>, body: String) -> FydiaResult {
    let json = get_json_value_from_body(&body)?;

    let email = get_json("email", &json)?;
    let password = get_json("password", &json)?;

    let mut user = User::by_email_and_password(email, password, &state.database)
        .await
        .ok_or_else(|| "User not exists".into_error())?;

    user.update_token(&state.database).await?;

    let token = user.token.ok_or_else(|| "Token error".into_error())?;

    Ok(token.into_ok())
}
