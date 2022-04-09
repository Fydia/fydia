use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::{
    impls::{channel::SqlChannel, message::SqlMessage, server::SqlMember},
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

pub async fn delete_message<'a>(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
    Extension(_rsa): Extension<Arc<RsaData>>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &executor,
    )
    .await?;

    let mut message = Message::get_message_by_id(&messageid, &executor)
        .await
        .map_err(FydiaResponse::StringError)?;

    if message.author_id.id != user.id {
        return Err(FydiaResponse::TextError("You can't delete this message"));
    }

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
                .map_err(FydiaResponse::StringError)?
                .to_userinfo(&executor)
                .await
                .map_err(|_| FydiaResponse::TextError("Can't delete"))?,
        )
        .await
        .map_err(|_| {
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot delete message",
            )
        })?;

    message
        .delete_message(&executor)
        .await
        .map_err(FydiaResponse::StringError)?;

    Ok(FydiaResponse::Text("Message delete"))
}
