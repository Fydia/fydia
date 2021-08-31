use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::channel::{Channel, ChannelType};
use fydia_struct::pathextractor::ServerExtractor;
use fydia_struct::server::{Server, ServerId};
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};
use serde_json::Value;

pub async fn create_channel(mut state: State) -> HandlerResult {
    let database = &SqlPool::borrow_from(&state).get_pool();
    let headers = HeaderMap::borrow_from(&state);
    let _token = headers.get("token").unwrap().to_str().unwrap().to_string();

    let body = String::from_utf8(
        body::to_bytes(Body::take_from(&mut state))
            .await
            .expect("Error")
            .to_vec(),
    )
    .unwrap();
    let serverid = ServerExtractor::borrow_from(&state);
    let value = serde_json::from_str::<Value>(body.as_str()).unwrap();

    let mut server =
        Server::get_server_by_id(ServerId::new(serverid.serverid.to_string()), database)
            .await
            .unwrap();

    let mut channel = Channel::new();

    channel.server_id = ServerId::new(server.id.clone());
    channel.name = value
        .get("name")
        .expect("Can't get name")
        .as_str()
        .unwrap()
        .to_string();
    channel.channel_type = ChannelType::from_string(
        value
            .get("type")
            .expect("Can't get type")
            .as_str()
            .unwrap()
            .to_string(),
    );

    server.insert_channel(channel.clone(), database).await;

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::TEXT_PLAIN_UTF_8,
        channel.id.clone(),
    );
    Ok((state, res))
}
