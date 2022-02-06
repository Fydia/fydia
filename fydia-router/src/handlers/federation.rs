use std::sync::Arc;

use crate::handlers::api::manager::websockets::manager::WebsocketManagerChannel;
use axum::body::Bytes;
use axum::extract::Extension;
use fydia_dispatcher::keys::get::get_public_key;
use fydia_dispatcher::message::receive::receive_message;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::messages::{Date, Message, MessageType};
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use http::HeaderMap;

pub async fn event_handler(
    headers: HeaderMap,
    body: Bytes,
    Extension(rsa): Extension<Arc<RsaData>>,
    Extension(database): Extension<DbConnection>,
    Extension(wbsockets): Extension<Arc<WebsocketManagerChannel>>,
) -> FydiaResult {
    let rsa = &rsa;
    let body = body.to_vec();
    let msg = receive_message(&headers, body, rsa)
        .await
        .ok_or_else(|| FydiaResponse::new_error("Decryption Error"))?;

    let event = serde_json::from_str::<Event>(msg.as_str())
        .map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    crate::handlers::event::event_handler(event, &database, &wbsockets).await;

    Ok(FydiaResponse::new_ok(""))
}

pub async fn send_test_message(Extension(keys): Extension<Arc<RsaData>>) -> FydiaResult {
    let event = Event::new(
        ServerId::new("1ENwYDlsoepW9HHZEmYxEl9KKRQFBD"),
        EventContent::Message {
            content: Box::from(
                Message::new(
                    String::from("This is a new message"),
                    MessageType::TEXT,
                    false,
                    Date::now(),
                    User::default(),
                    ChannelId::new("CkFg9d9IVf7Shht"),
                )
                .map_err(FydiaResponse::new_error)?,
            ),
        },
    );

    let publickey = get_public_key(Instance {
        protocol: fydia_struct::instance::Protocol::HTTP,
        domain: String::from("localhost"),
        port: 8080,
    })
    .await
    .ok_or_else(|| FydiaResponse::new_error("Cannot get the publickey"))?;

    fydia_dispatcher::message::send::send_message(
        &keys,
        Instance::new(fydia_struct::instance::Protocol::HTTP, "localhost", 8080),
        publickey,
        event,
        vec![Instance::new(
            fydia_struct::instance::Protocol::HTTP,
            "localhost",
            8080,
        )],
    )
    .await
    .map(|_| FydiaResponse::new_ok(""))
    .map_err(|_| FydiaResponse::new_error("Error when send message"))
}
