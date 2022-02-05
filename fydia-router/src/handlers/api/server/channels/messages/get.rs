use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

pub async fn get_message(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    channel
        .get_messages(&database)
        .await
        .map(|value| FydiaResponse::new_ok_json(&value))
        .map_err(|_| FydiaResponse::new_error("Cannot get message"))
}
