use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Extension, Path},
};

use fydia_sql::{
    impls::{channel::SqlChannel, message::SqlMessage, server::SqlMember},
    sqlpool::DbConnection,
};
use fydia_struct::{
    event::EventContent,
    instance::RsaData,
    messages::{Message, MessageType},
    response::{FydiaResponse, FydiaResult},
};
use http::{HeaderMap, StatusCode};

use crate::handlers::{
    api::manager::websockets::manager::{WbManagerChannelTrait, WebsocketManagerChannel},
    basic::BasicValues,
    get_json, get_json_value_from_body,
};

/// Change content of a message
///
/// # Errors
/// Return an error if :
/// * channelid, serverid isn't valid
/// * body isn't valid
pub async fn update_message<'a>(
    body: Bytes,
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

    if message.message_type != MessageType::TEXT && message.message_type != MessageType::URL {
        return Err(FydiaResponse::TextError("Cannot edit this type of message"));
    }

    if message.author_id.id != user.id {
        return Err(FydiaResponse::TextError("You can't edit this message"));
    }

    let value = get_json_value_from_body(&body).map_err(FydiaResponse::StringError)?;

    let content = get_json("content", &value)?.to_string();

    message
        .update_message(&content, &executor)
        .await
        .map_err(FydiaResponse::StringError)?;

    let userinfos = &channel
        .get_user_of_channel(&executor)
        .await
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot get user")
        })?
        .to_userinfo(&executor)
        .await
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot post message")
        })?;

    wbsocket
        .send(
            &fydia_struct::event::Event {
                server_id: server.id,
                content: EventContent::MessageUpdate {
                    message_id: messageid,
                    update: Box::new(message),
                },
            },
            userinfos,
        )
        .await
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot delete message",
            )
        })?;

    Ok(FydiaResponse::Text("Message delete"))
}
