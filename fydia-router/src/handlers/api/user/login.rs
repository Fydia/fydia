use crate::handlers::basic::{Database, UserFromJson};

use fydia_sql::impls::user::SqlUser;
use fydia_struct::response::{FydiaResult, IntoFydia};

/// Return a token
///
/// # Errors
/// This function return an error if body isn't valid or if user isn't exists
pub async fn user_login(
    Database(database): Database,
    UserFromJson(mut user): UserFromJson,
) -> FydiaResult {
    user.update_token(&database).await?;

    let token = user.token.ok_or_else(|| "Token error".into_error())?;

    Ok(token.into_ok())
}
