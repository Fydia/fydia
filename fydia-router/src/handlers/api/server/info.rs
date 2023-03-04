use crate::handlers::basic::UserFromToken;
use fydia_struct::response::{FydiaResponse, FydiaResult};

/// Return all server of user
///
/// # Errors
/// Return an error if the token isn't valid
pub async fn get_server_of_user(UserFromToken(user): UserFromToken) -> FydiaResult {
    Ok(FydiaResponse::from_serialize(user.servers))
}
