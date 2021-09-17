use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::error::FydiaResponse;
use fydia_struct::pathextractor::ChannelExtractor;
use gotham::{
    handler::HandlerResult,
    helpers::http::response::create_response,
    hyper::StatusCode,
    state::{FromState, State},
};

pub async fn delete_channel(state: State) -> HandlerResult {
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();

    if let Some(channel) = Channel::get_channel_by_id(
        ChannelId::new(channel_extracted.channelid.clone()),
        database,
    )
    .await
    {
        if let Err(error) = channel.delete_channel(database).await {
            error!(error);
            FydiaResponse::new_error(error).update_response(&mut res);
        };
    };

    Ok((state, res))
}
