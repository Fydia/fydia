use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};
use crate::handlers::basic::BasicValues;
use axum::body::Bytes;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
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
use fydia_struct::response::FydiaResponse;
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
) -> impl IntoResponse {
    let (user, server, channel) =
        match BasicValues::get_user_and_server_and_check_if_joined_and_channel(
            &headers, serverid, channelid, &database,
        )
        .await
        {
            Ok(v) => v,
            Err(error) => return error,
        };
    if let Some(headervalue) = headers.get(CONTENT_TYPE) {
        match headervalue.to_str() {
            Ok(value) => {
                return match Mime::from_str(value)
                    .map_err(|_| FydiaResponse::new_error("Bad Content-Type"))
                {
                    Ok(get_mime) => {
                        if get_mime == mime::APPLICATION_JSON
                            || get_mime == mime::TEXT_PLAIN
                            || get_mime == mime::TEXT_PLAIN_UTF_8
                            || value == "application/json; charset=utf-8"
                        {
                            let body = match String::from_utf8(body.to_vec())
                                .map_err(|_| FydiaResponse::new_error("Body error"))
                            {
                                Ok(v) => v,
                                Err(error) => return error,
                            };

                            let json = match serde_json::from_str(&body) {
                                Ok(v) => v,
                                Err(_) => {
                                    return FydiaResponse::new_error("Can't parse body");
                                }
                            };

                            post_messages_json(
                                json,
                                database,
                                rsa,
                                wbsocket,
                                (user, channel, server),
                            )
                            .await
                        } else if get_mime == mime::MULTIPART_FORM_DATA {
                            let stream = once(async move { Result::<Bytes, Infallible>::Ok(body) });
                            let boundary = match get_boundary(&headers) {
                                Some(v) => v,
                                None => return FydiaResponse::new_error("No boundary found"),
                            };
                            let multer = multer::Multipart::new(stream, boundary.clone());
                            post_messages_multipart(
                                multer,
                                database,
                                rsa,
                                wbsocket,
                                (user, channel, server),
                            )
                            .await
                        } else {
                            FydiaResponse::new_error("Content-Type error")
                        }
                    }
                    Err(error) => error,
                };
            }
            _ => return FydiaResponse::new_error("Content-Type error"),
        }
    }

    FydiaResponse::new_error("No Content-Type header found")
}

pub async fn post_messages_multipart(
    multipart: Multipart<'static>,
    database: DbConnection,
    rsa: Arc<RsaData>,
    wbsocket: Arc<WebsocketManagerChannel>,
    (user, channel, server): (User, Channel, Server),
) -> FydiaResponse {
    let event = match multipart_to_event(multipart, &user.clone(), &channel.id, &server.id).await {
        Ok(v) => v,
        Err(error) => return error,
    };
    let key = rsa.clone();
    if let Ok(members) = server.get_user(&database).await {
        if let EventContent::Message { ref content } = event.content {
            if content.insert_message(&database).await.is_err() {
                return FydiaResponse::new_error_custom_status(
                    "Cannot send message",
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            } else {
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
        }
        return FydiaResponse::new_ok("Message send");
    }

    FydiaResponse::new_error_custom_status(
        "Cannot get users of the server",
        StatusCode::INTERNAL_SERVER_ERROR,
    )
}

pub async fn post_messages_json(
    value: Value,
    database: DbConnection,
    rsa: Arc<RsaData>,
    wbsocket: Arc<WebsocketManagerChannel>,
    (user, channel, server): (User, Channel, Server),
) -> FydiaResponse {
    let event = match json_message(value, &user, &channel.id, &server.id).await {
        Ok(v) => v,
        Err(error) => return error,
    };
    let key = rsa.clone();
    if let Ok(members) = server.get_user(&database).await {
        if let EventContent::Message { ref content } = event.content {
            if content.insert_message(&database).await.is_err() {
                return FydiaResponse::new_error_custom_status(
                    "Cannot send message",
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }

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
        return FydiaResponse::new_ok("Message send");
    }

    FydiaResponse::new_error_custom_status(
        "Cannot get users of the server",
        StatusCode::INTERNAL_SERVER_ERROR,
    )
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

                let body = match field.bytes().await {
                    Ok(v) => v.to_vec(),
                    Err(_) => return Err(FydiaResponse::new_error("Body error")),
                };

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
            content: Box::from(Message::new(
                file.get_name(),
                MessageType::FILE,
                false,
                Date::new(DateTime::from(SystemTime::now())),
                user.clone(),
                channelid.clone(),
            )),
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
    let message_type;
    let content;

    match (value.get("type"), value.get("content")) {
        (Some(ctype), Some(mcontent)) => match (ctype.as_str(), mcontent.as_str()) {
            (Some(ctype), Some(mcontent)) => {
                message_type = ctype.to_string();
                content = mcontent.to_string();
            }
            _ => {
                return Err(FydiaResponse::new_error("Json error"));
            }
        },
        _ => {
            return Err(FydiaResponse::new_error("Json error"));
        }
    }

    if let Some(messagetype) = MessageType::from_string(message_type) {
        let event = Event::new(
            server_id.clone(),
            EventContent::Message {
                content: Box::from(Message::new(
                    content,
                    messagetype,
                    false,
                    Date::new(DateTime::from(SystemTime::now())),
                    user.clone(),
                    channelid.clone(),
                )),
            },
        );

        return Ok(event);
    }

    Err(FydiaResponse::new_error("Bad Message Type"))
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
