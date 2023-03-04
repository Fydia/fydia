pub mod create;
pub mod delete;
pub mod messages;
pub mod permission;
pub mod typing;
pub mod update;
pub mod vocal;

use fydia_struct::response::{FydiaResponse, FydiaResult};

use crate::handlers::basic::ChannelFromId;

/// Return requested channel
///
/// # Errors
/// Return an error if channelid isn't valid
pub async fn info_channel(ChannelFromId(channel): ChannelFromId) -> FydiaResult {
    Ok(FydiaResponse::from_serialize(channel))
}
