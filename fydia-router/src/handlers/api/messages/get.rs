use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::impls::server::SqlServerId;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::pathextractor::ChannelExtractor;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::ServerId;
use fydia_struct::user::Token;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{HeaderMap, StatusCode};
use gotham::state::{FromState, State};

pub async fn get_message(state: State) -> HandlerResult {
    let mut res = create_response(
        &state,
        StatusCode::BAD_REQUEST,
        mime::APPLICATION_JSON,
        "".to_string(),
    );
    let headers = HeaderMap::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let extracted = ChannelExtractor::borrow_from(&state);
    let serverid = extracted.serverid.clone();
    let channelid = extracted.channelid.clone();
    let token = if let Some(token) = Token::from_headervalue(headers) {
        token
    } else {
        return Ok((state, res));
    };

    if let Some(user) = token.get_user(database).await {
        if user.server.is_join(ServerId::new(serverid.clone())) {
            if let Some(serverid) = user.server.get(serverid) {
                if let Ok(server) = serverid.get_server(database).await {
                    if let Some(e) = server.channel.get_channel(channelid) {
                        if let Ok(message) = &e.get_messages(database).await {
                            FydiaResponse::new_ok_json(&message).update_response(&mut res);
                        }
                    }
                }
            }
        }
    }

    Ok((state, res))
}
