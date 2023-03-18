use crate::handlers::basic::UserFromToken;
use fydia_struct::response::{FydiaResponse, FydiaResult};

/// Get info of user
///
/// # Errors
/// This function will return an error if the token is wrong
pub async fn get_info_of_self(UserFromToken(user): UserFromToken) -> FydiaResult {
    let value = user.self_json_output()?;

    FydiaResponse::from_serialize(value).into()
}
