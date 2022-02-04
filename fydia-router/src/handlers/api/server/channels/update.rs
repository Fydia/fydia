use axum::body::Bytes;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;

use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;

pub async fn update_name(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (_, _, mut channel) =
        match BasicValues::get_user_and_server_and_check_if_joined_and_channel(
            &headers, serverid, channelid, &database,
        )
        .await
        {
            Ok(v) => v,
            Err(error) => return error,
        };

    if let Ok(body) = String::from_utf8(body.to_vec()) {
        if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
            if let Some(name) = value.get("name") {
                if let Some(name_str) = name.as_str() {
                    return if let Err(error) = channel.update_name(name_str, &database).await {
                        error!(error);
                        FydiaResponse::new_error("Cannot update description")
                    } else {
                        FydiaResponse::new_ok("Channel name updated")
                    };
                }
            };
        }
    }

    FydiaResponse::new_error("Json error")
}

pub async fn update_description(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (_, _, mut channel) =
        match BasicValues::get_user_and_server_and_check_if_joined_and_channel(
            &headers, serverid, channelid, &database,
        )
        .await
        {
            Ok(v) => v,
            Err(error) => return error,
        };

    if let Ok(body) = String::from_utf8(body.to_vec()) {
        if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
            if let Some(description) = value.get("description") {
                if let Some(description_str) = description.as_str() {
                    return if let Err(error) =
                        channel.update_description(description_str, &database).await
                    {
                        error!(error);
                        FydiaResponse::new_error("Cannot update description")
                    } else {
                        FydiaResponse::new_ok("Channel description updated")
                    };
                }
            }
        }
    }

    FydiaResponse::new_error("Json error")
}
