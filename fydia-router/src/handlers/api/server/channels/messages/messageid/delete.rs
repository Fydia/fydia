use std::sync::Arc;

use axum::extract::{Extension, Path};
use fydia_sql::{
    impls::{channel::SqlChannel, message::SqlMessage, user::SqlUser},
    sqlpool::DbConnection,
};
use fydia_struct::{
    event::EventContent,
    instance::RsaData,
    messages::Message,
    response::{FydiaResponse, FydiaResult},
};
use fydia_utils::http::{HeaderMap, StatusCode};

use crate::handlers::{
    api::manager::websockets::manager::{WbManagerChannelTrait, WebsocketManagerChannel},
    basic::BasicValues,
};

/// Delete a requested message
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, messageid, token isn't valid
/// * The owner user and token user is different
pub async fn delete_message<'a>(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
    Extension(_rsa): Extension<Arc<RsaData>>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &executor,
    )
    .await?;

    if !user
        .permission_of_channel(&channel.id, &executor)
        .await
        .map_err(|_| FydiaResponse::TextError("Cannot get permission"))?
        .calculate(Some(channel.id.clone()))
        .map_err(FydiaResponse::StringError)?
        .can_read()
    {
        return FydiaResult::Err(FydiaResponse::TextError("Unknow channel"));
    }

    let message = Message::by_id(&messageid, &executor)
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
                .users(&executor)
                .await
                .map_err(FydiaResponse::StringError)?,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot delete message",
            )
        })?;

    message
        .delete(&executor)
        .await
        .map_err(FydiaResponse::StringError)?;

    Ok(FydiaResponse::Text("Message delete"))
}
