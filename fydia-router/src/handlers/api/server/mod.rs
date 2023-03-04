use crate::handlers::basic::ServerJoinedFromId;
use fydia_struct::response::{FydiaResponse, FydiaResult};

pub mod channels;
pub mod create;
pub mod info;
pub mod join;
pub mod picture;
pub mod roles;

/// Return requested server
///
/// # Errors
/// This function will return if the token or serverid isn't valid
pub async fn get_server(ServerJoinedFromId(server): ServerJoinedFromId) -> FydiaResult {
    Ok(FydiaResponse::from_serialize(server))
}
