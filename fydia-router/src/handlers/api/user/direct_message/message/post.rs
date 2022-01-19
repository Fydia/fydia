use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{channel::ChannelId, messages::Message, response::FydiaResponse};
use http::{HeaderMap, StatusCode};

pub async fn post_message_dm(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    println!(
        "{:?}",
        Message::get_messages_by_channel(ChannelId::new(dm_id.clone()), &database).await
    );

    FydiaResponse::new_error_custom_status("", StatusCode::NOT_IMPLEMENTED)
}
