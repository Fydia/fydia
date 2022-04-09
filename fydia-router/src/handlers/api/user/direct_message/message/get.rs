use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};

use http::{HeaderMap, StatusCode};

pub async fn get_message_dm<'a>(
    _headers: HeaderMap,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let message = Message::get_messages_by_channel(ChannelId::new(dm_id.clone()), &database).await;
    println!("{:?}", message);

    Err(FydiaResponse::TextErrorWithStatusCode(
        StatusCode::NOT_IMPLEMENTED,
        "",
    ))
}
