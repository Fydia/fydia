use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use fydia_utils::http::{HeaderMap, StatusCode};

/// Send a new message in dm
///
/// # Errors
/// This function will return an error if dm isn't exists
pub async fn post_message_dm<'a>(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    println!(
        "{:?}",
        Message::by_channel(ChannelId::new(dm_id.clone()), &database).await
    );

    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "",
    ))
}
