use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::error::FydiaResponse;
use fydia_struct::pathextractor::ChannelExtractor;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, StatusCode};
use gotham::state::{FromState, State};
use serde_json::Value;

pub async fn update_name(mut state: State) -> HandlerResult {
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));
    let body = body::to_bytes(Body::take_from(&mut state));
    if let Ok(body_bytes) = body.await {
        let body_vec = body_bytes.to_vec();
        if let Ok(body) = String::from_utf8(body_vec) {
            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                let channel_extracted = ChannelExtractor::borrow_from(&state);
                let database = &SqlPool::borrow_from(&state).get_pool();
                if let Some(name) = value.get("name") {
                    if let Some(name_str) = name.as_str() {
                        if let Some(mut channel) = Channel::get_channel_by_id(
                            ChannelId::new(channel_extracted.channelid.clone()),
                            database,
                        )
                        .await
                        {
                            if let Err(error) =
                                channel.update_name(name_str.to_string(), database).await
                            {
                                error!(error);
                                FydiaResponse::new_error("Cannot update description")
                                    .update_response(&mut res);
                            }
                        }
                    }
                };
            }
        }
    }

    Ok((state, res))
}

pub async fn update_description(mut state: State) -> HandlerResult {
    let mut res = create_response(&state, StatusCode::OK, mime::TEXT_PLAIN_UTF_8, format!(""));

    let body = body::to_bytes(Body::take_from(&mut state));
    let channel_extracted = ChannelExtractor::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();

    if let Ok(body_bytes) = body.await {
        let body = body_bytes.to_vec();
        if let Ok(body) = String::from_utf8(body) {
            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                if let Some(description) = value.get("description") {
                    if let Some(description_str) = description.as_str() {
                        if let Some(mut channel) = Channel::get_channel_by_id(
                            ChannelId::new(channel_extracted.channelid.clone()),
                            database,
                        )
                        .await
                        {
                            if let Err(error) = channel
                                .update_description(description_str.to_string(), database)
                                .await
                            {
                                error!(error);
                                FydiaResponse::new_error("Cannot update description")
                                    .update_response(&mut res);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((state, res))
}
