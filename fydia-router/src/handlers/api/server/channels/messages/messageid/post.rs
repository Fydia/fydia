use axum::extract::{Path, State};

use fydia_sql::impls::{channel::SqlChannel, message::SqlMessage, user::SqlUser};
use fydia_struct::{
    event::EventContent,
    messages::{Message, MessageType},
    response::{FydiaResult, IntoFydia, MapError},
};
use fydia_utils::http::HeaderMap;

use crate::{
    handlers::{
        api::manager::websockets::manager::WbManagerChannelTrait, basic::BasicValues, get_json,
        get_json_value_from_body,
    },
    ServerState,
};

/// Change content of a message
///
/// # Errors
/// Return an error if :
/// * channelid, serverid isn't valid
/// * body isn't valid
pub async fn update_message(
    State(state): State<ServerState>,
    Path((serverid, channelid, messageid)): Path<(String, String, String)>,
    headers: HeaderMap,
    body: String,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers,
        &serverid,
        &channelid,
        &state.database,
    )
    .await?;

    if !user
        .permission_of_channel(&channel.id, &state.database)
        .await?
        .calculate(Some(channel.id.clone()))
        .error_to_fydiaresponse()?
        .can_read()
    {
        return Err("Unknow channel".into_error());
    }

    let mut message = Message::by_id(&messageid, &state.database).await?;

    if message.message_type != MessageType::TEXT && message.message_type != MessageType::URL {
        return Err("Cannot edit this type of message".into_error());
    }

    if message.author_id.id != user.id {
        return Err("You can't edit this message".into_error());
    }

    let value = get_json_value_from_body(&body)?;

    let content = get_json("content", &value)?.to_string();

    message.update(&content, &state.database).await?;

    let users = &channel.users(&state.database).await?;

    let _ = &state
        .wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageUpdate {
                    message_id: messageid,
                    update: Box::new(message),
                },
            },
            users,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            "Cannot delete message".into_server_error()
        })?;

    Ok("Message delete".into_ok())
}
