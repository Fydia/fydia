#![allow(clippy::unwrap_used)]

use crate::handlers::api::websocket::ChannelMessage;
use crate::new_response;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Query};
use axum::response::IntoResponse;
use futures::prelude::*;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::querystring::QsToken;
use fydia_struct::user::{Token, User};
use http::StatusCode;
use serde::Serialize;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::WebsocketManagerChannel;

pub async fn ws_handler(
    Extension(_): Extension<DbConnection>,
    Extension(wbsocket): Extension<WebsocketManagerChannel>,
    Query(token): Query<QsToken>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let _res = new_response();
    let _token = Token(token.token.unwrap_or_default());
    //let user = token.get_user(&database).await;
    let user = User::default();
    let server = unbounded_channel::<ChannelMessage>();
    wbsocket.insert_channel(user.clone(), server.0.clone());
    ws.on_upgrade(|e| _connected(e, wbsocket, user, server))
        .into_response()
}

pub fn empty_response() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, "")
}

async fn _connected(
    socket: WebSocket,
    wbmanager: WebsocketManagerChannel,
    user: User,
    channel: (
        UnboundedSender<ChannelMessage>,
        UnboundedReceiver<ChannelMessage>,
    ),
) {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut receiver) = channel;
    let thread_sender = sender.clone();
    tokio::spawn(async move {
        let sender = thread_sender;
        while let Some(Ok(e)) = stream.next().await {
            println!("{:?}", e.to_text());
            match e {
                Message::Text(str) => {
                    if let Err(e) = sender
                        .clone()
                        .send(ChannelMessage::WebsocketMessage(Message::Text(str)))
                    {
                        error!(e.to_string());
                    };
                }
                Message::Binary(bin) => {
                    if let Err(e) = sender
                        .clone()
                        .send(ChannelMessage::WebsocketMessage(Message::Binary(bin)))
                    {
                        error!(e.to_string());
                    };
                }
                Message::Ping(ping) => {
                    if let Err(e) = sender
                        .clone()
                        .send(ChannelMessage::WebsocketMessage(Message::Ping(ping)))
                    {
                        error!(e.to_string());
                    };
                }
                Message::Pong(pong) => {
                    if let Err(e) = sender
                        .clone()
                        .send(ChannelMessage::WebsocketMessage(Message::Pong(pong)))
                    {
                        error!(e.to_string());
                    };
                }
                Message::Close(_) => {
                    if let Err(e) = sender.clone().send(ChannelMessage::Kill) {
                        error!(e.to_string());
                    };
                }
            };
        }
    });
    let thread_sender = sender;
    tokio::spawn(async move {
        while let Some(channelmessage) = receiver.recv().await {
            println!("{:?}", channelmessage);
            match channelmessage {
                ChannelMessage::WebsocketMessage(e) => match sink.send(e).await {
                    Ok(_) => {}
                    Err(e) => println! {"{}", e},
                },
                ChannelMessage::Message(e) => match sink
                    .send(axum::extract::ws::Message::Text(format!(
                        "{:?}",
                        serde_json::to_string(&e)
                    )))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        println!("{}", e.to_string());
                    }
                },
                ChannelMessage::Kill => wbmanager
                    .clone()
                    .remove_channel_of_user(user.clone(), &thread_sender.clone()),
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
