pub mod create;
pub mod delete;
pub mod update;
pub mod vocal;

use gotham::{
    handler::HandlerResult,
    helpers::http::response::create_response,
    hyper::StatusCode,
    state::{FromState, State},
};

use fydia_sql::{impls::channel::SqlChannel, sqlpool::SqlPool};

use fydia_struct::{
    channel::{Channel, ChannelId},
    pathextractor::ChannelExtractor,
    response::FydiaResponse,
};

pub async fn info_channel(state: State) -> HandlerResult {
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, "");
    if let Some(channel) = Channel::get_channel_by_id(
        ChannelId::new(channel_extracted.channelid.clone()),
        database,
    )
    .await
    {
        FydiaResponse::new_ok_json(&channel).update_response(&mut res);
    } else {
        FydiaResponse::new_error("Error").update_response(&mut res);
    }
    Ok((state, res))
}
