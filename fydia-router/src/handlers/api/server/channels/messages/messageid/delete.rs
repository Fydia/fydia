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
    response::{FydiaResult, IntoFydia, MapError},
};
use fydia_utils::http::HeaderMap;

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
        .await?
        .calculate(Some(channel.id.clone()))
        .error_to_fydiaresponse()?
        .can_read()
    {
        return FydiaResult::Err("Unknow channel".into_error());
    }

    let message = Message::by_id(&messageid, &executor).await?;

    if message.author_id.id != user.id {
        return Err("You can't delete this message".into_error());
    }

    wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageDelete {
                    message_id: messageid,
                },
            },
            &channel.users(&executor).await?,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            "Cannot delete message".into_server_error()
        })?;

    message.delete(&executor).await?;

    Ok("Message delete".into_error())
}
