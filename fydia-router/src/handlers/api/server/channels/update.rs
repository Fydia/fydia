use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::pathextractor::ChannelExtractor;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, StatusCode};
use gotham::state::{FromState, State};
use serde_json::Value;

pub async fn update_name(mut state: State) -> HandlerResult {
    let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));

    let body = String::from_utf8(
        body::to_bytes(Body::take_from(&mut state))
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();

    let value = serde_json::from_str::<Value>(body.as_str()).unwrap();

    let name = value.get("name").unwrap().as_str().unwrap().to_string();

    Channel::get_channel_by_id(
        ChannelId::new(channel_extracted.channelid.clone()),
        database,
    )
    .await
    .expect("Error")
    .update_name(name, database)
    .await;

    Ok((state, res))
}

pub async fn update_description(mut state: State) -> HandlerResult {
    let res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));

    let body = String::from_utf8(
        body::to_bytes(Body::take_from(&mut state))
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();

    let value = serde_json::from_str::<Value>(body.as_str()).unwrap();

    let description = value
        .get("description")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    Channel::get_channel_by_id(
        ChannelId::new(channel_extracted.channelid.clone()),
        database,
    )
    .await
    .expect("Error")
    .update_description(description, database)
    .await;

    Ok((state, res))
}
