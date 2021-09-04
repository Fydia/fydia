pub mod create;
pub mod delete;
pub mod update;
pub mod vocal;

use gotham::{
    handler::{HandlerError, HandlerResult},
    helpers::http::response::create_response,
    hyper::StatusCode,
    state::{FromState, State},
};

use fydia_sql::{impls::channel::SqlChannel, sqlpool::SqlPool};

use fydia_struct::{
    channel::{Channel, ChannelId},
    pathextractor::ChannelExtractor,
};

pub async fn info_channel(state: State) -> HandlerResult {
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();

    if let Some(channel) = Channel::get_channel_by_id(
        ChannelId::new(channel_extracted.channelid.clone()),
        database,
    )
    .await
    {
        match serde_json::to_string(&channel) {
            Ok(json) => {
                let res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, json);
                Ok((state, res))
            }
            Err(error) => Err((state, HandlerError::from(anyhow::Error::new(error)))),
        }
    } else {
        let res = create_response(&state, StatusCode::BAD_REQUEST, mime::APPLICATION_JSON, "");
        Ok((state, res))
    }
}
