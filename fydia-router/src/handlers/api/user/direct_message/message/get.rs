use axum::extract::Path;
use fydia_sql::impls::message::SqlMessage;
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResult, IntoFydia},
};

use fydia_utils::http::HeaderMap;

use crate::handlers::basic::Database;

/// Get messages of a dm
///
/// # Errors
/// This function will return an error if dm does not exist
pub async fn get_message_dm(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Database(database): Database,
) -> FydiaResult {
    let message = Message::by_channel(ChannelId::new(dm_id.clone()), &database).await;
    println!("{:?}", message);

    Err("".into_not_implemented_error())
}
