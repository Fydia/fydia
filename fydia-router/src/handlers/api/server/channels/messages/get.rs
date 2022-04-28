use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

/// Return all message of channel
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn get_messages<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    channel
        .get_messages(&database)
        .await
        .map(|value| FydiaResponse::from_serialize(&value))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot get message")
        })
}
