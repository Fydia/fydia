use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn delete_channel(
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;
    channel
        .delete_channel(&database)
        .await
        .map(|_| FydiaResponse::new_ok("Channel deleted"))
        .map_err(|error| {
            error!(error);
            FydiaResponse::new_error(error)
        })
}
