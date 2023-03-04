use fydia_sql::impls::{channel::SqlChannel, message::SqlMessage};
use fydia_struct::{
    event::EventContent,
    response::{FydiaResult, IntoFydia},
};

use crate::handlers::{
    api::manager::websockets::manager::WbManagerChannelTrait,
    basic::{
        ChannelFromId, Database, MessageFromId, ServerJoinedFromId, UserFromToken, WebsocketManager,
    },
};

/// Delete a requested message
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, messageid, token isn't valid
/// * The owner user and token user is different
pub async fn delete_message(
    Database(database): Database,
    UserFromToken(user): UserFromToken,
    ServerJoinedFromId(server): ServerJoinedFromId,
    ChannelFromId(channel): ChannelFromId,
    MessageFromId(message): MessageFromId,
    WebsocketManager(wbsocket): WebsocketManager,
) -> FydiaResult {
    if message.author_id.id != user.id {
        return Err("You can't delete this message".into_error());
    }

    wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageDelete {
                    message_id: message.id.clone(),
                },
            },
            &channel.users(&database).await?,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            "Cannot delete message".into_server_error()
        })?;

    message.delete(&database).await?;

    Ok("Message delete".into_error())
}
