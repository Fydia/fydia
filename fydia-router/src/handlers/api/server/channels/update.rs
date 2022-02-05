use axum::body::Bytes;
use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};

use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;
use crate::handlers::get_json;

pub async fn update_name(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (_, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    let body =
        String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let json =
        serde_json::from_str::<Value>(&body).map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let name = get_json("name", &json)?;

    channel
        .update_name(name, &database)
        .await
        .map(|_| FydiaResponse::new_ok("Channel name updated"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::new_error("Cannot update name")
        })
}

pub async fn update_description(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (_, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    let body =
        String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let json =
        serde_json::from_str::<Value>(&body).map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let description = get_json("description", &json)?;

    channel
        .update_description(description, &database)
        .await
        .map(|_| FydiaResponse::new_ok("Channel description updated"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::new_error("Cannot update description")
        })
}
