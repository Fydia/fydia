use fydia_sql::impls::channel::SqlChannelId;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::{channel::ChannelId, pathextractor::ChannelExtractor, server::ServerId};
use gotham::state::FromState;
use gotham::{
    handler::HandlerResult, helpers::http::response::create_response, hyper::HeaderMap,
    state::State,
};
use reqwest::StatusCode;

pub async fn join_channel(state: State) -> HandlerResult {
    let headers = HeaderMap::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let extracted = ChannelExtractor::borrow_from(&state);
    let serverid = ServerId::new(extracted.serverid.clone());
    let channelid = ChannelId::new(extracted.channelid.clone());
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, "");

    if let Some(channel) = channelid.get_channel(database).await {
        if channel.channel_type.is_voice() {
            *res.body_mut() = "Vocal Channel".into()
        } else {
            *res.body_mut() = "text Channel".into()
        }
    };

    Ok((state, res))
}
