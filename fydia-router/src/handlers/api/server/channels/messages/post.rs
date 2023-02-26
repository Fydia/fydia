use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};
use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};
use crate::ServerState;

use axum::extract::{Path, State};
use chrono::DateTime;
use futures::stream::once;
use fydia_sql::impls::message::SqlMessage;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::file::{File, FileDescriptor};
use fydia_struct::instance::RsaData;
use fydia_struct::messages::{Date, Message, MessageType};
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia, MapError};
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::User;
use fydia_utils::http::header::CONTENT_TYPE;
use fydia_utils::http::HeaderMap;
use fydia_utils::serde_json::Value;
use mime::Mime;
use multer::Multipart;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

const BOUNDARY: &str = "boundary=";
const CHECK_MIME: [mime::Mime; 3] = [
    mime::APPLICATION_JSON,
    mime::TEXT_PLAIN,
    mime::TEXT_PLAIN_UTF_8,
];

/// Post a new messages in a channel
///
/// # Errors
/// Return an error if:
/// * Channelid, Serverid isn't valid
/// * Body is bad
/// * Content-Type isn't valid
pub async fn post_messages(
    State(state): State<ServerState>,
    Path((serverid, channelid)): Path<(String, String)>,
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

    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or_else(|| "No Content-Type header found".into_error())?
        .to_str()
        .map_err(|error| {
            error!("{error}");
            "Content-Type error".into_error()
        })?;

    let mime_type = Mime::from_str(content_type).map_err(|error| {
        error!("{error}");
        "Bad Content-Type".into_error()
    })?;

    if CHECK_MIME.contains(&mime_type) || content_type == "application/json; charset=utf-8" {
        let json = get_json_value_from_body(&body)?;
        let event = json_message(json, user, &channel.id, &server.id).await?;
        return send_event(event, server, &state.rsa, state.wbsocket, state.database).await;
    }

    if mime_type == mime::MULTIPART_FORM_DATA {
        let stream = once(async move { Result::<String, Infallible>::Ok(body) });
        let boundary = get_boundary(&headers).ok_or_else(|| "No boundary found".into_error())?;

        let multer = multer::Multipart::new(stream, boundary.clone());

        let event = multipart_to_event(multer, user.clone(), &channel.id, &server.id).await?;

        return send_event(event, server, &state.rsa, state.wbsocket, state.database).await;
    }

    Err("Content-Type error".into_error())
}

/// Transform a multipart request to a Message event
///
/// # Errors
/// Return an error if:
/// * body isn't valid
/// * Cannot write file
pub async fn multipart_to_event<'a, 'r>(
    mut multipart: Multipart<'r>,
    user: User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    let file = File::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            if field_name == "file" {
                let file = File::new();

                file.create_with_description(&FileDescriptor::new_with_now(
                    field
                        .file_name()
                        .map_or_else(|| file.get_name(), |v| v.to_string()),
                ))
                .map_err(|error| {
                    error!("{error}");
                    "File creation error".into_server_error()
                })?;

                let body = field
                    .bytes()
                    .await
                    .map_err(|error| {
                        error!("{error}");
                        "Body error".into_error()
                    })?
                    .to_vec();

                file.write(&body).map_err(|error| {
                    error!("{error}");

                    "Can't write the file".into_server_error()
                })?;

                break;
            } else if field_name.eq("context") {
                info!("{}", field.text().await.unwrap_or_default());
            }
        }
    }

    let message = Message::new(
        file.get_name(),
        MessageType::FILE,
        false,
        Date::new(DateTime::from(SystemTime::now())),
        user,
        channelid.clone(),
    )
    .error_to_fydiaresponse()?;

    let event = Event::new(
        server_id.clone(),
        EventContent::Message {
            content: Box::from(message),
        },
    );

    Ok(event)
}

/// Get message from value
///
/// # Errors
/// Return an error if:
/// * The body isn't valid
/// * The channelid, serverid isn't valid
pub async fn json_message(
    value: Value,
    user: User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    let type_from_json = get_json("type", &value)?.to_string();
    let messagetype =
        MessageType::from_string(type_from_json).ok_or_else(|| "Bad Message Type".into_error())?;

    let content = get_json("content", &value)?.to_string();

    let message = Message::new(
        content,
        messagetype,
        false,
        Date::new(DateTime::from(SystemTime::now())),
        user,
        channelid.clone(),
    )
    .error_to_fydiaresponse()?;

    Ok(Event::new(
        server_id.clone(),
        EventContent::Message {
            content: Box::from(message),
        },
    ))
}

pub fn get_mime_of_file(path: &str) -> Mime {
    let mut buf = [0; 16];

    let file = File::get(path);

    if file.read_file(&mut buf).is_err() {
        error!("Can't write on buf");
    }

    if let Some(e) = infer::get(&buf) {
        return mime::Mime::from_str(e.mime_type()).unwrap_or(mime::APPLICATION_OCTET_STREAM);
    }

    if let Ok(file_desc) = file.get_description() {
        return mime_guess::from_path(file_desc.name).first_or_octet_stream();
    }

    mime::APPLICATION_OCTET_STREAM
}

pub fn get_boundary(headers: &HeaderMap) -> Option<String> {
    headers.get(CONTENT_TYPE).and_then(|ct| {
        let ct = ct.to_str().ok()?;
        let idx = ct.find(BOUNDARY)?;
        Some(ct[idx + BOUNDARY.len()..].to_string())
    })
}

/// Send message event
///
/// # Errors
/// Return error if :
/// * cannot get members of server
/// * cannot get websocket manager
pub async fn send_event(
    event: Event,
    server: Server,
    rsa: &Arc<RsaData>,
    wbsocket: Arc<WebsocketManagerChannel>,
    database: DbConnection,
) -> FydiaResult {
    let members = match server.users(&database).await {
        Ok(members) => members.members,
        Err(_) => return Err("Cannot get users of the server".into_server_error()),
    };

    if let EventContent::Message { ref content } = event.content {
        if content.insert(&database).await.is_err() {
            return Err("Cannot send message".into_server_error());
        }

        let key = rsa.clone();
        tokio::spawn(async move {
            if let Err(error) = wbsocket
                .send_with_origin_and_key(&event, members.as_slice(), Some(&key), None)
                .await
            {
                error!("{error}");
            };
        });
    }

    Ok("Message send".into_error())
}
