use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResult, IntoFydia};
use fydia_utils::http::HeaderMap;

use crate::handlers::{
    api::manager::typing::{TypingManagerChannel, TypingManagerChannelTrait},
    basic::BasicValues,
};

/// Start typing
///
/// # Errors
/// Return an error if typingmanager is unreachable
/// or if serverid, channelid or the token isn't valid
pub async fn start_typing(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    typingmanager
        .start_typing(user.id, channel.id, server.id)
        .map(|_| "".into_ok())
        .map_err(|error| {
            error!("{error}");
            "Can't start typing".into_server_error()
        })
}

/// Stop typing
///
/// # Errors
/// Return an error if typingmanager is unreachable
/// or if serverid, channelid or the token isn't valid
pub async fn stop_typing(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    typingmanager
        .stop_typing(user.id, channel.id, server.id)
        .map(|_| "".into_ok())
        .map_err(|error| {
            error!("{error}");
            "Can't stop typing".into_server_error()
        })
}
