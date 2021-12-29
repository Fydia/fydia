use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use fydia_sql::impls::channel::{SqlChannel, SqlChannelId};
use fydia_sql::impls::server::SqlServerId;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::server::ServerId;
use fydia_struct::user::{Token, UserId};
use http::{HeaderMap, StatusCode};

use crate::handlers::api::manager::{
    typing::{TypingManagerChannel, TypingManagerChannelTrait},
    websockets::manager::WebsocketManagerChannel,
};

pub async fn start_typing(
    Extension(database): Extension<DbConnection>,
    Extension(typingmanager): Extension<Arc<TypingManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            if let Ok(server) = ServerId::new(&serverid).get_server(&database).await {
                if let Some(channel) = ChannelId::new(&channelid).get_channel(&database).await {
                    if let Ok(users) = channel.get_user_of_channel(&database).await {
                        typingmanager.start_typing(
                            UserId::new(user.id),
                            ChannelId::new(channel.id),
                            ServerId::new(server.id),
                            users,
                        );
                    }
                }
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
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            if ServerId::new(&serverid).get_server(&database).await.is_ok() {
                if let Some(channel) = ChannelId::new(&channelid).get_channel(&database).await {
                    if let Ok(users) = channel.get_user_of_channel(&database).await {
                        if let Err(error) = typingmanager.stop_typing(
                            UserId::new(user.id),
                            ChannelId::new(channelid),
                            ServerId::new(serverid),
                            users,
                        ) {
                            error!(error);
                        };
                    }
                }
            }
        }
    }
    (StatusCode::OK, "")
}
