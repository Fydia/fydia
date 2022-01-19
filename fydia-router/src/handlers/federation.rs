use std::sync::Arc;

use crate::handlers::api::manager::websockets::manager::WebsocketManagerChannel;
use axum::body::Bytes;
use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_dispatcher::keys::get::get_public_key;
use fydia_dispatcher::message::receive::receive_message;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use http::HeaderMap;

use crate::new_response;

pub async fn event_handler(
    headers: HeaderMap,
    body: Bytes,
    Extension(rsa): Extension<Arc<RsaData>>,
    Extension(database): Extension<DbConnection>,
    Extension(wbsockets): Extension<Arc<WebsocketManagerChannel>>,
) -> impl IntoResponse {
    let rsa = &rsa;
    let mut res = new_response();
    let body = body.to_vec();

    if let Some(msg) = receive_message(&headers, body, rsa).await {
        if let Ok(event) = serde_json::from_str::<Event>(msg.as_str()) {
            crate::handlers::event::event_handler(event, &database, &wbsockets).await;
        } else {
            FydiaResponse::new_error("Bad Body").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("Decryption Error").update_response(&mut res);
    }

    res
}

pub async fn send_test_message(Extension(keys): Extension<Arc<RsaData>>) -> impl IntoResponse {
    let event = Event::new(
        ServerId::new("1ENwYDlsoepW9HHZEmYxEl9KKRQFBD".to_string()),
        EventContent::Message {
            content: Box::from(Message::new(
                String::from("This is a new message"),
                MessageType::TEXT,
                false,
                SqlDate::now(),
                User::default(),
                ChannelId::new("CkFg9d9IVf7Shht".to_string()),
            )),
        },
    );

    let mut res = new_response();

    if let Some(publickey) = get_public_key(Instance {
        protocol: fydia_struct::instance::Protocol::HTTP,
        domain: String::from("localhost"),
        port: 8080,
    })
    .await
    {
        if fydia_dispatcher::message::send::send_message(
            &keys,
            Instance::new(
                fydia_struct::instance::Protocol::HTTP,
                "localhost".to_string(),
                8080,
            ),
            publickey,
            event,
            vec![Instance::new(
                fydia_struct::instance::Protocol::HTTP,
                "localhost".to_string(),
                8080,
            )],
        )
        .await
        .is_err()
        {
            FydiaResponse::new_error("Error when send message").update_response(&mut res);
        };
    };

    res
}
