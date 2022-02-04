use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannel;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn delete_channel(
    headers: HeaderMap,
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let (_, _, channel) = match BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await
    {
        Ok(v) => v,
        Err(error) => return error,
    };

    if let Err(error) = channel.delete_channel(&database).await {
        error!(error);
        FydiaResponse::new_error(error)
    } else {
        FydiaResponse::new_ok("Channel deleted")
    }
}
