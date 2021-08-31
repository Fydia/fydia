use fydia_dispatcher::keys::get::get_public_key;
use fydia_dispatcher::message::receive::receive_message;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub async fn event_handler(mut state: State) -> HandlerResult {
    let body = body::to_bytes(Body::take_from(&mut state))
        .await
        .expect("Error")
        .to_vec();
    let headers = HeaderMap::borrow_from(&state);
    let rsa = RsaData::borrow_from(&state);
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));

    if let Some(msg) = receive_message(headers, body, rsa).await {
        if let Ok(event) = serde_json::from_str::<Event>(msg.as_str()) {
            crate::handlers::event::event_handler(event, &mut state).await;
        } else {
            *res.body_mut() = "Bad Body".into();
            *res.status_mut() = StatusCode::BAD_REQUEST;
        }
    } else {
        *res.body_mut() = "Decryption error".into();
        *res.status_mut() = StatusCode::BAD_REQUEST;
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

    let publickey = get_public_key(Instance {
        protocol: fydia_struct::instance::Protocol::HTTP,
        domain: String::from("localhost"),
        port: 8080,
    })
    .await
    .unwrap();

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
        *res.body_mut() = "Error when send message".into()
    };

    Ok((state, res))
}
