use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use http::{HeaderMap, StatusCode};

pub async fn post_message_dm(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    println!(
        "{:?}",
        Message::get_messages_by_channel(ChannelId::new(dm_id.clone()), &database).await
    );

    Err(FydiaResponse::new_error_custom_status(
        "",
        StatusCode::NOT_IMPLEMENTED,
    ))
}
