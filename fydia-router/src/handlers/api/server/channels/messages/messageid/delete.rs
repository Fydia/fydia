use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::{
    impls::{channel::SqlChannel, message::SqlMessage},
    sqlpool::DbConnection,
};
use fydia_struct::{
    event::EventContent,
    instance::RsaData,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use http::{HeaderMap, StatusCode};

use crate::handlers::{
    api::manager::websockets::manager::{WbManagerChannelTrait, WebsocketManagerChannel},
    basic::BasicValues,
};

pub async fn delete_message(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
    Extension(_rsa): Extension<Arc<RsaData>>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &executor,
    )
    .await?;

    let mut message = Message::get_message_by_id(&messageid, &executor)
        .await
        .map_err(FydiaResponse::new_error)?;
    if message.author_id.id != user.id {
        return Err(FydiaResponse::new_error("You can't delete this message"));
    }
    message
        .delete_message(&executor)
        .await
        .map_err(FydiaResponse::new_error)?;

    wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageDelete {
                    message_id: messageid,
                },
            },
            &channel
                .get_user_of_channel(&executor)
                .await
                .map_err(FydiaResponse::new_error)?,
        )
        .await
        .map_err(|_| {
            FydiaResponse::new_error_custom_status(
                "Cannot delete message",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    Ok(FydiaResponse::new_ok("Message delete"))
}
