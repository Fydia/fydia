use axum::body::Body;
use axum::extract::{BodyStream, Extension, Path};
use axum::http::Request;
use axum::response::IntoResponse;
use futures::StreamExt;
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::response::FydiaResponse;

use serde_json::Value;

use crate::new_response;

pub async fn update_name(
    mut body: BodyStream,
    Extension(database): Extension<DbConnection>,
    Path((_, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut res = new_response();

    while let Some(Ok(chuck)) = body.next().await {
        let body_vec = chuck.to_vec();
        if let Ok(body) = String::from_utf8(body_vec) {
            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                if let Some(name) = value.get("name") {
                    if let Some(name_str) = name.as_str() {
                        if let Some(mut channel) =
                            Channel::get_channel_by_id(ChannelId::new(channelid.clone()), &database)
                                .await
                        {
                            if let Err(error) =
                                channel.update_name(name_str.to_string(), &database).await
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

    res
}

pub async fn update_description(
    _request: Request<Body>,
    mut body: BodyStream,
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut res = new_response();

    while let Some(Ok(body_bytes)) = body.next().await {
        let body = body_bytes.to_vec();
        if let Ok(body) = String::from_utf8(body) {
            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                if let Some(description) = value.get("description") {
                    if let Some(description_str) = description.as_str() {
                        if let Some(mut channel) =
                            Channel::get_channel_by_id(ChannelId::new(channelid.clone()), &database)
                                .await
                        {
                            if let Err(error) = channel
                                .update_description(description_str.to_string(), &database)
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

    res
}
