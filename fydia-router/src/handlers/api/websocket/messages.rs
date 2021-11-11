#![allow(clippy::unwrap_used)]

use crate::handlers::api::websocket::{ChannelMessage, WbUser};
use crate::new_response;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Query};
use axum::response::IntoResponse;

use futures::prelude::*;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::event::Event;
use fydia_struct::instance::Instance;
use fydia_struct::querystring::QsToken;
use fydia_struct::user::{Token, User};
use http::StatusCode;
use logger::info;
use serde::Serialize;

use super::Websockets;

pub async fn ws_handler(
    Extension(_): Extension<DbConnection>,
    Extension(wbsocket): Extension<Websockets>,
    Query(token): Query<QsToken>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let _res = new_response();
    let _token = Token(token.token.clone().unwrap_or_default());
    //let user = token.get_user(&database).await;
    let user = Some(User::new("name", "email", "password", Instance::default()));
    let ws = ws
        .on_upgrade(|e| connected(e, user, wbsocket))
        .into_response();
    return ws;
}

pub fn empty_response() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, "")
}

async fn connected(
    socket: WebSocket,
    user: Option<User>,
    wbsockets_channel: Websockets,
) -> Result<String, String> {
    if let Some(user) = user {
        let mut user = user.clone();
        user.password = None;
        let (mut sink, mut stream) = socket.split();

        let wblist = &wbsockets_channel;
        let e = wblist.get_channels_clone().await.0;
        let wbuser = {
            let mut channels: Option<WbUser> = None;
            for i in e.iter() {
                println!("{:?}", i);
                if i.user.eq(&user) {
                    channels = Some(i.clone());
                }
            }

            if channels.is_none() {
                let (sender, receiver) = flume::unbounded::<ChannelMessage>();

                let mut id = rand::random::<u32>();

                e.iter().for_each(|e: &WbUser| {
                    if e.id == id {
                        id = rand::random::<u32>();
                    }
                });

                let wbuser = WbUser::new(id, (sender, receiver), user);
                wblist.insert(&wbuser).await;
                channels = Some(wbuser);
            }

            channels.unwrap()
        };
        println!("{:#?}", wbsockets_channel);
        let sendchannel = wbuser.channel.0.clone();
        let task = tokio::spawn(async move {
            while let Ok(Some(message)) = stream
                .next()
                .await
                .transpose()
                .map_err(|error| println!("Websocket receive error: {}", error))
            {
                let a = match message {
                    axum::extract::ws::Message::Text(string) => {
                        if let Ok(e) = serde_json::from_str::<Event>(string.as_str()) {
                            ChannelMessage::Message(Box::new(e))
                        } else {
                            ChannelMessage::WebsocketMessage(Message::Text(string))
                        }
                    }
                    axum::extract::ws::Message::Binary(value) => {
                        ChannelMessage::WebsocketMessage(Message::Binary(value))
                    }
                    axum::extract::ws::Message::Ping(value) => {
                        ChannelMessage::WebsocketMessage(Message::Ping(value))
                    }
                    axum::extract::ws::Message::Pong(value) => {
                        ChannelMessage::WebsocketMessage(Message::Pong(value))
                    }
                    axum::extract::ws::Message::Close(_) => ChannelMessage::Kill,
                };

                if sendchannel.send(a).is_err() {
                    error!("Can't send message");
                }
            }
        });

        while let Ok(msg) = wbuser.channel.1.recv() {
            match msg {
                ChannelMessage::WebsocketMessage(msg) => match sink.send(msg).await {
                    Ok(()) => {}
                    Err(error) => {
                        println!("{:?}", error);
                        return Err(error.to_string());
                    }
                },
                ChannelMessage::Kill => {
                    if sink.close().await.is_err() {
                        error!("Can't close channel");
                    };
                    break;
                }
                ChannelMessage::Message(msg) => {
                    if let Ok(msg) = to_websocketmessage(&msg) {
                        match sink.send(msg).await {
                            Ok(()) => {}
                            Err(error) => {
                                println!("{:?}", error);
                                if wblist.remove(wbuser.id).await.is_err() {
                                    info!("Can't Remove");
                                    return Err(String::from("Can't Remove"));
                                }
                                return Err(error.to_string());
                            }
                        }
                    }
                }
            }
        }
        task.abort();
        if wblist.remove(wbuser.id).await.is_err() {
            info!("Can't Remove");
            return Err("Can't Remove".to_string());
        }
        println!("Connection Closed");
    } else {
        return Err(String::from("Token Error"));
    }

    Ok(String::from("Connection finished"))
}

pub fn to_websocketmessage<T>(msg: &T) -> Result<Message, String>
where
    T: Serialize,
{
    match serde_json::to_string(msg) {
        Ok(json) => Ok(Message::from(axum::extract::ws::Message::Text(json))),
        Err(e) => Err(e.to_string()),
    }
}
