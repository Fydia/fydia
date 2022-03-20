use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{
    instance::RsaData,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn get_message(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
    Extension(_rsa): Extension<Arc<RsaData>>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
) -> FydiaResult {
    let (_, _, _) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &executor,
    )
    .await?;

    let message = Message::get_message_by_id(&messageid, &executor)
        .await
        .map_err(FydiaResponse::new_error)?;

    Ok(FydiaResponse::new_ok_json(&message))
}