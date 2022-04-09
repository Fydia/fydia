use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::{HeaderMap, StatusCode};

use crate::handlers::{
    api::manager::typing::{TypingManagerChannel, TypingManagerChannelTrait},
    basic::BasicValues,
};

pub async fn start_typing<'a>(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    typingmanager
        .start_typing(user.id, channel.id, server.id)
        .map(|_| FydiaResponse::Text(""))
        .map_err(|error| {
            error!(error);
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Can't start typing",
            )
        })
}
pub async fn stop_typing<'a>(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    typingmanager
        .stop_typing(user.id, channel.id, server.id)
        .map(|_| FydiaResponse::Text(""))
        .map_err(|error| {
            error!(error);
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Can't start typing",
            )
        })
}
