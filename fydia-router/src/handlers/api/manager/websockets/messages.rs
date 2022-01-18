#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use crate::handlers::api::manager::websockets::ChannelMessage;
use crate::new_response;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Query};
use axum::response::IntoResponse;
use futures::prelude::*;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::querystring::QsToken;
use fydia_struct::user::{Token, User};
use http::StatusCode;
use serde::Serialize;

use super::manager::{WbManagerChannelTrait, WebsocketManagerChannel};

pub async fn ws_handler(
    Extension(database): Extension<DbConnection>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    Query(token): Query<QsToken>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let _res = new_response();
    let token = Token(token.token.unwrap_or_default());
    let user = token.get_user(&database).await;
    ws.on_upgrade(move |e| connected(e, wbsocket, user))
        .into_response()
}

pub fn empty_response<'a>() -> (StatusCode, &'a str) {
    (StatusCode::BAD_REQUEST, "")
}

async fn connected(socket: WebSocket, wbmanager: Arc<WebsocketManagerChannel>, user: Option<User>) {
    let user = if let Some(user) = user { user } else { return };
    let channel = if let Some(channels) = wbmanager.get_new_channel(user.clone()).await {
        channels
    } else {
        return;
    };
    let (mut sink, mut stream) = socket.split();
    let (sender, mut receiver) = channel;
    let thread_sender = sender.clone();

    tokio::spawn(async move {
        let sender = thread_sender;
        while let Some(Ok(e)) = stream.next().await {
            if std::mem::discriminant(&e) == std::mem::discriminant(&Message::Close(None)) {
                if let Err(e) = sender.clone().send(ChannelMessage::Kill) {
                    error!(e.to_string());
                };
            } else if let Err(e) = sender.clone().send(ChannelMessage::WebsocketMessage(e)) {
                error!(e.to_string());
            };
        }
    });
    let sender = sender;
    tokio::spawn(async move {
        while let Some(channelmessage) = receiver.recv().await {
            match channelmessage {
                ChannelMessage::WebsocketMessage(e) => match sink.send(e).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!(e.to_string());
                    }
                },
                ChannelMessage::Message(e) => {
                    if let Ok(msg) = serde_json::to_string(&e) {
                        match sink.send(axum::extract::ws::Message::Text(msg)).await {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                }
                ChannelMessage::Kill => {
                    if wbmanager
                        .clone()
                        .remove(user.clone(), &sender)
                        .await
                        .is_err()
                    {
                        error!("Can't remove");
                    };
                    break;
                }
            }
        }
    });
}

pub fn to_websocketmessage<T>(msg: &T) -> Result<Message, String>
where
    T: Serialize,
{
    match serde_json::to_string(msg) {
        Ok(json) => Ok(axum::extract::ws::Message::Text(json)),
        Err(e) => Err(e.to_string()),
    }
}
