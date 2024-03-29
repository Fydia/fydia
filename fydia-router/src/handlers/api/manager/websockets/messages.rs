#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use crate::handlers::api::manager::websockets::ChannelMessage;
use crate::handlers::basic::{Database, WebsocketManager};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::response::IntoResponse;
use futures::prelude::*;
use fydia_sql::impls::token::SqlToken;
use fydia_struct::querystring::QsToken;
use fydia_struct::user::{Token, User, UserError};
use fydia_utils::{serde::Serialize, serde_json};

use super::manager::{WbManagerChannelTrait, WebsocketManagerChannel};

pub async fn ws_handler(
    Database(database): Database,
    WebsocketManager(wbsocket): WebsocketManager,
    Query(token): Query<QsToken>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let token = Token::new(token.token.unwrap_or_default());
    let user = token.get_user(&database).await;

    ws.on_upgrade(move |e| connected(e, wbsocket, user))
}

async fn connected(
    socket: WebSocket,
    wbmanager: Arc<WebsocketManagerChannel>,
    user: Result<User, UserError>,
) {
    let Ok(user) = user else {
        return;
    };

    let Some((sender, mut receiver)) = wbmanager.get_new_channel(&user.id).await else {
        return;
    };

    let (mut sink, mut stream) = socket.split();
    let thread_sender = sender.clone();

    tokio::spawn(async move {
        let sender = thread_sender;
        while let Some(Ok(e)) = stream.next().await {
            if std::mem::discriminant(&e) == std::mem::discriminant(&Message::Close(None)) {
                if let Err(e) = sender.send(ChannelMessage::Kill) {
                    error!("{e}");
                };
            } else if let Err(e) = sender.send(ChannelMessage::WebsocketMessage(e)) {
                error!("{e}");
            };
        }
    });
    let sender = sender;
    tokio::spawn(async move {
        while let Some(channelmessage) = receiver.recv().await {
            match channelmessage {
                ChannelMessage::WebsocketMessage(e) => {
                    if let Err(error) = sink.send(e).await {
                        error!("{error}");
                    }
                }
                ChannelMessage::Message(e) => {
                    let json = serde_json::to_string(&e);

                    if json.is_err() {
                        error!("{:?}", json);
                    }

                    if let Ok(msg) = json {
                        if let Err(error) = sink.send(Message::Text(msg)).await {
                            error!("{error}");
                        }
                    }
                }
                ChannelMessage::Kill => {
                    if wbmanager.remove(&user.id, &sender).await.is_err() {
                        error!("Can't remove");
                    };
                    break;
                }
            }
        }
    });
}

/// Convert a json to Websocket Message
///
/// # Errors
/// Return an error if:
/// * serialize isn't possible
pub fn to_websocketmessage<T>(msg: &T) -> Result<Message, String>
where
    T: Serialize,
{
    serde_json::to_string(msg)
        .map(axum::extract::ws::Message::Text)
        .map_err(|error| error.to_string())
}
