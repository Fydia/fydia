use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn join_channel(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let (_, _, channel) = match BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await
    {
        Ok(v) => v,
        Err(error) => return FydiaResponse::new_error(error),
    };

    if channel.channel_type.is_voice() {
        FydiaResponse::new_ok("Vocal Channel")
    } else {
        FydiaResponse::new_ok("Text Channel")
    }
}
