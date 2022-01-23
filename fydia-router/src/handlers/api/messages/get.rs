use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

pub async fn get_message(
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
        Err(error) => return error,
    };

    if let Ok(message) = &channel.get_messages(&database).await {
        return FydiaResponse::new_ok_json(&message);
    }

    FydiaResponse::new_error("Cannot get message")
}
