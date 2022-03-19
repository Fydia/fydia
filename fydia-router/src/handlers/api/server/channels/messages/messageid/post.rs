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
    get_json,
};

pub async fn update_message(
    body: Bytes,
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

    if message.message_type != MessageType::TEXT && message.message_type != MessageType::URL {
        return Err(FydiaResponse::new_error("Cannot edit this type of message"));
    }

    if message.author_id.id != user.id {
        return Err(FydiaResponse::new_error("You can't edit this message"));
    }

    let body =
        String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Body error"))?;

    let value = serde_json::from_str(&body).map_err(|_| FydiaResponse::new_error("JSON error"))?;

    let content = get_json("content", &value)?.to_string();

    message
        .update_message(&content, &executor)
        .await
        .map_err(FydiaResponse::new_error)?;

    let userinfos = &channel
        .get_user_of_channel(&executor)
        .await
        .map_err(FydiaResponse::new_error)?
        .to_userinfo(&executor)
        .await
        .map_err(|_| FydiaResponse::new_error("Cannot post message"))?;

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
        .map_err(|_| {
            FydiaResponse::new_error_custom_status(
                "Cannot delete message",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    Ok(FydiaResponse::new_ok("Message delete"))
}
