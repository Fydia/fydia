use fydia_sql::impls::{channel::SqlChannel, message::SqlMessage};
use fydia_struct::{
    event::EventContent,
    messages::MessageType,
    response::{FydiaResult, IntoFydia},
};

use crate::handlers::{
    api::manager::websockets::manager::WbManagerChannelTrait,
    basic::{
        ChannelFromId, Database, MessageFromId, ServerFromId, UserFromToken, WebsocketManager,
    },
    get_json, get_json_value_from_body,
};

/// Change content of a message
///
/// # Errors
/// Return an error if :
/// * channelid, serverid isn't valid
/// * body isn't valid
pub async fn update_message(
    UserFromToken(user): UserFromToken,
    ServerFromId(server): ServerFromId,
    ChannelFromId(channel): ChannelFromId,
    MessageFromId(mut message): MessageFromId,
    Database(database): Database,
    WebsocketManager(wbsocket): WebsocketManager,
    body: String,
) -> FydiaResult {
    if message.message_type != MessageType::TEXT && message.message_type != MessageType::URL {
        return "Cannot edit this type of message".into();
    }

    if message.author_id.id != user.id {
        return "You can't edit this message".into();
    }

    let value = get_json_value_from_body(&body)?;

    let content = get_json("content", &value)?.to_string();

    message.update(&content, &database).await?;

    let users = &channel.users(&database).await?;

    wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageUpdate {
                    message_id: message.id.clone(),
                    update: Box::new(message),
                },
            },
            users,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            "Cannot edit message".into_server_error()
        })?;

    "Message edited".into()
}
