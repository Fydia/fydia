use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    instance::RsaData,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Return requested message
///
/// # Errors
/// Return error if:
/// * serverid, channelid, messageid, token isn't valid
pub async fn get_message<'a>(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
    Extension(_rsa): Extension<Arc<RsaData>>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
) -> FydiaResult<'a> {
    let (_, _, _) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &executor,
    )
    .await?;

    let message = Message::by_id(&messageid, &executor)
        .await
        .map_err(FydiaResponse::StringError)?;

    Ok(FydiaResponse::from_serialize(&message))
}
