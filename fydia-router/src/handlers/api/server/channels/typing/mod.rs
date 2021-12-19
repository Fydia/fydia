use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use fydia_sql::impls::channel::{SqlChannel, SqlChannelId};
use fydia_sql::impls::server::SqlServerId;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::server::ServerId;
use fydia_struct::user::{Token, UserId};
use http::{HeaderMap, StatusCode};
use tokio::spawn;

use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};

pub async fn start_typing(
    Extension(database): Extension<DbConnection>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            if let Ok(_) = ServerId::new(&serverid).get_server(&database).await {
                if let Some(channel) = ChannelId::new(&channelid).get_channel(&database).await {
                    println!("{:#?}", channel);
                    if let Ok(users) = channel.get_user_of_channel(&database).await {
                        println!("{:#?}", users);
                        let event = Event::new(
                            ServerId::new(&serverid),
                            EventContent::StartTyping {
                                userid: UserId::new(user.id),
                                channelid: ChannelId::new(&channelid),
                            },
                        );
                        wbsocket
                            .send(event, users.clone(), None, None)
                            .await
                            .unwrap();

                        spawn(async move {
                            sleep(Duration::from_secs(5));
                            let event = Event::new(
                                ServerId::new(&serverid),
                                EventContent::StopTyping {
                                    userid: UserId::new(user.id),
                                    channelid: ChannelId::new(&channelid),
                                },
                            );
                            wbsocket.send(event, users, None, None).await.unwrap();
                        });
                    }
                }
            }
        }
    }

    (StatusCode::OK, "")
}
pub async fn stop_typing(
    Extension(database): Extension<DbConnection>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&database).await {
            if ServerId::new(&serverid).get_server(&database).await.is_ok() {
                if let Some(channel) = ChannelId::new(&channelid).get_channel(&database).await {
                    if let Ok(users) = channel.get_user_of_channel(&database).await {
                        let event = Event::new(
                            ServerId::new(&serverid),
                            EventContent::StopTyping {
                                userid: UserId::new(user.id),
                                channelid: ChannelId::new(&channelid),
                            },
                        );
                        wbsocket.send(event, users, None, None).await.unwrap();
                    }
                }
            }
        }
    }
    (StatusCode::OK, "")
}
