use axum::body::Bytes;
use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};

use http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};

/// Change name of a channel
///
/// # Errors
/// Return an error if serverid or channelid or body isn't valid
pub async fn update_name<'a>(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (_, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    let json = get_json_value_from_body(&body).map_err(|error| {
        error!("{error}");
        FydiaResponse::StringError(error)
    })?;

    let name = get_json("name", &json)?;

    channel
        .update_name(name, &database)
        .await
        .map(|_| FydiaResponse::Text("Channel name updated"))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot update name")
        })
}

/// Change description of a channel
///
/// # Errors
/// Return an error if channelid or serverid or body isn't valid
pub async fn update_description<'a>(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (_, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    let json = get_json_value_from_body(&body).map_err(|error| {
        error!("{error}");
        FydiaResponse::StringError(error)
    })?;

    let description = get_json("description", &json)?;

    channel
        .update_description(description, &database)
        .await
        .map(|_| FydiaResponse::Text("Channel description updated"))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot update description")
        })
}
