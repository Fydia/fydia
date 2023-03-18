use crate::handlers::basic::ChannelFromId;
use fydia_struct::response::FydiaResult;

/// Join a vocal channel
///
/// # Errors
/// Return an error if channelid isn't valid or if channel is text
pub async fn join_channel(ChannelFromId(channel): ChannelFromId) -> FydiaResult {
    if channel.channel_type.is_voice() {
        return "Vocal Channel".into();
    }

    "Text Channel".into()
}
