use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};
use crate::handlers::basic::BasicValues;
use crate::handlers::get_json;
use axum::body::Bytes;
use axum::extract::{Extension, Path};
use chrono::DateTime;
use futures::stream::once;
use fydia_sql::impls::message::SqlMessage;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::event::{Event, EventContent};
use fydia_struct::file::{File, FileDescriptor};
use fydia_struct::instance::RsaData;
use fydia_struct::messages::{Date, Message, MessageType};
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::User;
use http::header::CONTENT_TYPE;
use http::{HeaderMap, StatusCode};
use mime::Mime;
use multer::Multipart;
use serde_json::Value;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

const BOUNDARY: &str = "boundary=";

pub async fn post_messages(
    body: Bytes,
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Extension(rsa): Extension<Arc<RsaData>>,
    Extension(wbsocket): Extension<Arc<WebsocketManagerChannel>>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, serverid, channelid, &database,
    )
    .await?;

    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or_else(|| FydiaResponse::new_error("No Content-Type header found"))?
        .to_str()
        .map_err(|_| FydiaResponse::new_error("Content-Type error"))?;

    let mime_type =
        Mime::from_str(content_type).map_err(|_| FydiaResponse::new_error("Bad Content-Type"))?;

    if mime_type == mime::APPLICATION_JSON
        || mime_type == mime::TEXT_PLAIN
        || mime_type == mime::TEXT_PLAIN_UTF_8
        || content_type == "application/json; charset=utf-8"
    {
        let body =
            String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Body error"))?;

        let json =
            serde_json::from_str(&body).map_err(|_| FydiaResponse::new_error("JSON error"))?;

        post_messages_json(json, database, rsa, wbsocket, (user, channel, server)).await
    } else if mime_type == mime::MULTIPART_FORM_DATA {
        let stream = once(async move { Result::<Bytes, Infallible>::Ok(body) });
        let boundary =
            get_boundary(&headers).ok_or_else(|| FydiaResponse::new_error("No boundary found"))?;

        let multer = multer::Multipart::new(stream, boundary.clone());

        post_messages_multipart(multer, database, rsa, wbsocket, (user, channel, server)).await
    } else {
        Err(FydiaResponse::new_error("Content-Type error"))
    }
}

pub async fn post_messages_multipart(
    multipart: Multipart<'static>,
    database: DbConnection,
    rsa: Arc<RsaData>,
    wbsocket: Arc<WebsocketManagerChannel>,
    (user, channel, server): (User, Channel, Server),
) -> FydiaResult {
    let event = multipart_to_event(multipart, &user.clone(), &channel.id, &server.id).await?;
    let key = rsa.clone();
    let members = server.get_user(&database).await.map_err(|_| {
        FydiaResponse::new_error_custom_status(
            "Cannot get users of the server",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    if let EventContent::Message { ref content } = event.content {
        content.insert_message(&database).await.map_err(|_| {
            FydiaResponse::new_error_custom_status(
                "Cannot send message",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        tokio::spawn(async move {
            if wbsocket
                .send_with_origin_and_key(event, members.members.clone(), Some(&key), None)
                .await
                .is_err()
            {
                error!("Error");
            };
        });
    }

    Ok(FydiaResponse::new_ok("Message send"))
}

pub async fn post_messages_json(
    value: Value,
    database: DbConnection,
    rsa: Arc<RsaData>,
    wbsocket: Arc<WebsocketManagerChannel>,
    (user, channel, server): (User, Channel, Server),
) -> FydiaResult {
    let event = json_message(value, &user, &channel.id, &server.id).await?;
    let key = rsa.clone();
    let members = server.get_user(&database).await.map_err(|_| {
        FydiaResponse::new_error_custom_status(
            "Cannot get users of the server",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    if let EventContent::Message { ref content } = event.content {
        content.insert_message(&database).await.map_err(|_| {
            FydiaResponse::new_error_custom_status(
                "Cannot send message",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        tokio::spawn(async move {
            if wbsocket
                .send_with_origin_and_key(event, members.members.clone(), Some(&key), None)
                .await
                .is_err()
            {
                error!(r#"Error"#);
            };
        });
    }

    Ok(FydiaResponse::new_ok("Message send"))
}

pub async fn multipart_to_event(
    mut multipart: Multipart<'static>,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    let file = File::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(field_name) = field.name() {
            if field_name == "file" {
                let file = File::new();
                file.create_with_description(FileDescriptor::new_with_now(
                    field
                        .file_name()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| file.get_name()),
                ))
                .map_err(|_| {
                    FydiaResponse::new_error_custom_status(
                        "File creation error",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;
                let body = field
                    .bytes()
                    .await
                    .map_err(|_| FydiaResponse::new_error("Body error"))?
                    .to_vec();

                file.write(body).map_err(|_| {
                    FydiaResponse::new_error_custom_status(
                        "Can't write the file",
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?;

                break;
            } else if field_name.eq("context") {
                info!(field.text().await.unwrap_or_default());
            }
        }
    }

    let event = Event::new(
        server_id.clone(),
        EventContent::Message {
            content: Box::from(
                Message::new(
                    file.get_name(),
                    MessageType::FILE,
                    false,
                    Date::new(DateTime::from(SystemTime::now())),
                    user.clone(),
                    channelid.clone(),
                )
                .map_err(|error| FydiaResponse::new_error(error))?,
            ),
        },
    );

    Ok(event)
}

pub async fn json_message(
    value: Value,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    let messagetype = MessageType::from_string(get_json("type", &value)?.to_string())
        .ok_or_else(|| FydiaResponse::new_error("Bad Message Type"))?;
    let content = get_json("content", &value)?.to_string();

    Ok(Event::new(
        server_id.clone(),
        EventContent::Message {
            content: Box::from(
                Message::new(
                    content,
                    messagetype,
                    false,
                    Date::new(DateTime::from(SystemTime::now())),
                    user.clone(),
                    channelid.clone(),
                )
                .map_err(|error| FydiaResponse::new_error(error))?,
            ),
        },
    ))
}

pub fn get_mime_of_file(path: &str) -> Mime {
    let mut buf = [0; 16];

    let file = File::get(path);
    if file.write_value(&mut buf).is_err() {
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
