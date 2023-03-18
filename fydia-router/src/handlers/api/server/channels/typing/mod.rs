use fydia_struct::response::{FydiaResult, IntoFydia};

use crate::handlers::{
    api::manager::typing::TypingManagerChannelTrait,
    basic::{ChannelFromId, ServerJoinedFromId, TypingManager, UserFromToken},
};

/// Start typing
///
/// # Errors
/// Return an error if typingmanager is unreachable
/// or if serverid, channelid or the token isn't valid
pub async fn start_typing(
    UserFromToken(user): UserFromToken,
    ChannelFromId(channel): ChannelFromId,
    ServerJoinedFromId(server): ServerJoinedFromId,
    TypingManager(typing): TypingManager,
) -> FydiaResult {
    typing
        .start_typing(user.id, channel.id, server.id)
        .map_err(|error| {
            error!("{error}");
            "Can't start typing".into_server_error()
        })?;

    "".into()
}

/// Stop typing
///
/// # Errors
/// Return an error if typingmanager is unreachable
/// or if serverid, channelid or the token isn't valid
pub async fn stop_typing(
    UserFromToken(user): UserFromToken,
    ChannelFromId(channel): ChannelFromId,
    ServerJoinedFromId(server): ServerJoinedFromId,
    TypingManager(typing): TypingManager,
) -> FydiaResult {
    typing
        .stop_typing(user.id, channel.id, server.id)
        .map_err(|error| {
            error!("{error}");
            "Can't stop typing".into_server_error()
        })?;

    "".into()
}
