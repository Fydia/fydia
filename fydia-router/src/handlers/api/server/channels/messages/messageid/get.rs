use fydia_struct::response::{FydiaResponse, FydiaResult};

use crate::handlers::basic::{ChannelFromId, MessageFromId};

/// Return requested message
///
/// # Errors
/// Return error if:
/// * serverid, channelid, messageid, token isn't valid
pub async fn get_message(
    ChannelFromId(_): ChannelFromId,
    MessageFromId(message): MessageFromId,
) -> FydiaResult {
    Ok(FydiaResponse::from_serialize(message))
}
