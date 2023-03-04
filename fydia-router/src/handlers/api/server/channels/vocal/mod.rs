use crate::handlers::basic::ChannelFromId;
use fydia_struct::response::{FydiaResult, IntoFydia};

/// Join a vocal channel
///
/// # Errors
/// Return an error if channelid isn't valid or if channel is text
pub async fn join_channel(ChannelFromId(channel): ChannelFromId) -> FydiaResult {
    if channel.channel_type.is_voice() {
        return Ok("Vocal Channel".into_ok());
    }
    Err("Text Channel".into_error())
}
