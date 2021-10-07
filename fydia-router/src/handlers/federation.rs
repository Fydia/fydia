use fydia_dispatcher::keys::get::get_public_key;
use fydia_dispatcher::message::receive::receive_message;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub async fn event_handler(mut state: State) -> HandlerResult {
    let body = body::to_bytes(Body::take_from(&mut state));
    let headers = HeaderMap::borrow_from(&state);
    let rsa = RsaData::borrow_from(&state);
    let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, "");
    if let Ok(body_bytes) = body.await {
        let body = body_bytes.to_vec();
        if let Some(msg) = receive_message(headers, body, rsa).await {
            if let Ok(event) = serde_json::from_str::<Event>(msg.as_str()) {
                crate::handlers::event::event_handler(event, &mut state).await;
            } else {
                FydiaResponse::new_error("Bad Body").update_response(&mut res);
            }
        } else {
            FydiaResponse::new_error("Decryption Error").update_response(&mut res);
        }
    }

    Ok((state, res))
}

pub async fn send_test_message(state: State) -> HandlerResult {
    let keys = RsaData::borrow_from(&state);
    let event = Event::new(
        ServerId::new("1ENwYDlsoepW9HHZEmYxEl9KKRQFBD".to_string()),
        EventContent::Message {
            content: Message::new(
                String::from("This is a new message"),
                MessageType::TEXT,
                false,
                SqlDate::now(),
                User::default(),
                ChannelId::new("CkFg9d9IVf7Shht".to_string()),
            ),
        },
    );

    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, format!(""));

    if let Some(publickey) = get_public_key(Instance {
        protocol: fydia_struct::instance::Protocol::HTTP,
        domain: String::from("localhost"),
        port: 8080,
    })
    .await
    {
        if fydia_dispatcher::message::send::send_message(
            keys,
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

    Ok((state, res))
}
