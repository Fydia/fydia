use axum::extract::{Extension, Path};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Join a vocal channel
///
/// # Errors
/// Return an error if channelid isn't valid or if channel is text
pub async fn join_channel<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    if channel.channel_type.is_voice() {
        Ok(FydiaResponse::Text("Vocal Channel"))
    } else {
        Err(FydiaResponse::TextError("Text Channel"))
    }
}
