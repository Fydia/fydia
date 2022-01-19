use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use http::{HeaderMap, StatusCode};

use crate::handlers::{
    api::manager::typing::{TypingManagerChannel, TypingManagerChannelTrait},
    basic::BasicValues,
};

pub async fn start_typing(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Ok((user, server, channel)) =
        BasicValues::get_user_and_server_and_check_if_joined_and_channel(
            &headers, serverid, channelid, &database,
        )
        .await
    {
        if let Ok(users) = channel.get_user_of_channel(&database).await {
            if let Err(error) = typingmanager.start_typing(user.id, channel.id, server.id, users) {
                error!(error);
            }
        }
    }

    (StatusCode::OK, "")
}
pub async fn stop_typing(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Ok((user, server, channel)) =
        BasicValues::get_user_and_server_and_check_if_joined_and_channel(
            &headers, serverid, channelid, &database,
        )
        .await
    {
        if let Ok(users) = channel.get_user_of_channel(&database).await {
            if let Err(error) = typingmanager.stop_typing(user.id, channel.id, server.id, users) {
                error!(error);
            }
        }
    }

    (StatusCode::OK, "")
}
