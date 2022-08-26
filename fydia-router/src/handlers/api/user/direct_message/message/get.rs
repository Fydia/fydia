use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResult, IntoFydia},
};

use fydia_utils::http::HeaderMap;

/// Get messages of a dm
///
/// # Errors
/// This function will return an error if dm does not exist
pub async fn get_message_dm<'a>(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let message = Message::by_channel(ChannelId::new(dm_id.clone()), &database).await;
    println!("{:?}", message);

    Err("".into_not_implemented_error())
}
