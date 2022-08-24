use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaMap, FydiaResponse, FydiaResult};
use fydia_utils::http::{HeaderMap, StatusCode};

/// Delete a channel in a server
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn delete_channel<'a>(
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    channel.delete(&database).await.fydia_map(
        |_| FydiaResponse::Text("Channel deleted"),
        |_| {
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot delete the channel",
            )
        },
    )
}
