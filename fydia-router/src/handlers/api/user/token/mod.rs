use fydia_struct::response::{FydiaResult, IntoFydia};

use crate::handlers::basic::UserFromToken;

/// Return a 200 OK if token is valid
///
/// # Errors
/// This function will return an error if token isn't valid
pub async fn verify(UserFromToken(_): UserFromToken) -> FydiaResult {
    Ok("".into_ok())
}
