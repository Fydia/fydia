use axum::extract::{Extension, Path};
use fydia_sql::impls::channel::SqlChannel;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::{HeaderMap, StatusCode};

use crate::handlers::basic::BasicValues;

pub async fn delete_channel<'a>(
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    channel
        .delete_channel(&database)
        .await
        .map(|_| FydiaResponse::Text("Channel deleted"))
        .map_err(|_| {
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot delete the channel (SQL Server Error)",
            )
        })
}
