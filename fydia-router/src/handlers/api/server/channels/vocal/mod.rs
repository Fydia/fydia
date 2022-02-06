use axum::extract::{Extension, Path};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn join_channel(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    if channel.channel_type.is_voice() {
        Ok(FydiaResponse::new_ok("Vocal Channel"))
    } else {
        Ok(FydiaResponse::new_ok("Text Channel"))
    }
}
