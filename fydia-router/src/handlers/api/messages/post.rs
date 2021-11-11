use crate::handlers::api::websocket::Websockets;
use crate::new_response;
use axum::body::{Body, Bytes};
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use chrono::DateTime;
use futures::stream::once;
use fydia_sql::impls::message::SqlMessage;
use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::RsaData;
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::ServerId;
use fydia_struct::user::{Token, User};
use fydia_utils::generate_string;
use http::header::CONTENT_TYPE;
use http::{HeaderMap, Request, StatusCode};
use mime::Mime;
use serde_json::Value;
use std::convert::Infallible;
use std::io::{Read, Write};
use std::str::FromStr;
use std::time::SystemTime;

const BOUNDARY: &str = "boundary=";

pub async fn post_messages(
    request: Request<Body>,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
    Extension(rsa): Extension<RsaData>,
    Extension(wbsocket): Extension<Websockets>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut res = new_response();

    let headers = request.headers();
    let serverid = ServerId::new(serverid.clone());

    let channelid = channelid.clone();
    let token = if let Some(token) = Token::from_headervalue(headers) {
        token
    } else {
        FydiaResponse::new_error("Bad Token").update_response(&mut res);
        return res;
    };

    if let Some(user) = token.get_user(&database).await {
        if user.server.is_join(serverid.clone()) {
            if let Some(serverid) = user.server.get(serverid.clone().short_id) {
                if let Ok(server) = serverid.get_server(&database).await {
                    if server.channel.is_exists(channelid.clone()) {
                        if let Some(header_content_type) = headers.get(CONTENT_TYPE) {
                            if let Ok(content_type) = header_content_type.to_str() {
                                let msg = match messages_dispatcher(
                                    content_type,
                                    body.to_vec(),
                                    headers,
                                    &user,
                                    &ChannelId::new(channelid),
                                    &serverid,
                                )
                                .await
                                {
                                    Ok(event) => event,
                                    Err(err) => {
                                        err.update_response(&mut res);
                                        return res;
                                    }
                                };
                                let mut websocket = wbsocket.get_channels().await.clone();
                                let key = rsa.clone();
                                if let Ok(members) = server.get_user(&database).await {
                                    if let EventContent::Message { ref content } = msg.content {
                                        if content.insert_message(&database).await.is_err() {
                                            FydiaResponse::new_error_custom_status(
                                                "Cannot send message",
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                            )
                                            .update_response(&mut res);
                                        } else {
                                            tokio::spawn(async move {
                                                if websocket
                                                    .send(
                                                        &msg.clone(),
                                                        members.members.clone(),
                                                        Some(&key),
                                                        None,
                                                    )
                                                    .await
                                                    .is_err()
                                                {
                                                    error!("Error");
                                                };
                                            });
                                        }
                                    }
                                    FydiaResponse::new_ok("Message send").update_response(&mut res);
                                } else {
                                    FydiaResponse::new_error_custom_status(
                                        "Cannot get users of the server",
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                    )
                                    .update_response(&mut res);
                                }
                            } else {
                                FydiaResponse::new_error("Bad Content-Type")
                                    .update_response(&mut res);
                            }
                        } else {
                            FydiaResponse::new_error("Where is the Content-Type")
                                .update_response(&mut res);
                        }
                    } else {
                        FydiaResponse::new_error("Unvalid channel").update_response(&mut res);
                    }
                } else {
                    FydiaResponse::new_error("unknow server").update_response(&mut res);
                }
            } else {
                FydiaResponse::new_error("unknow server").update_response(&mut res);
            }
        } else {
            FydiaResponse::new_error("unknow server").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("Token error").update_response(&mut res);
    }

    res
}

pub async fn messages_dispatcher(
    content_type: &str,
    body: Vec<u8>,
    headers: &HeaderMap,
    user: &User,
    channelid: &ChannelId,
    serverid: &ServerId,
) -> Result<Event, FydiaResponse> {
    match content_type {
        "application/json" | "application/json; charset=utf-8" => match String::from_utf8(body) {
            Ok(string_body) => json_message(string_body, user, channelid, serverid),
            Err(_) => Err(FydiaResponse::new_error("Utf-8 Error")),
        },

        "multipart/form-data" | "multipart/form-data;" => {
            multipart_to_event(body, headers, &user.clone(), channelid, serverid).await
        }

        _ => Err(FydiaResponse::new_error("Bad Content-Type")),
    }
}

pub async fn multipart_to_event(
    body: Vec<u8>,
    headers: &HeaderMap,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    if let Some(boundary) = headers.get(CONTENT_TYPE).and_then(|ct| {
        let ct = ct.to_str().ok()?;
        let idx = ct.find(BOUNDARY)?;
        Some(ct[idx + BOUNDARY.len()..].to_string())
    }) {
        let a = once(async move {
            Result::<Bytes, Infallible>::Ok(Bytes::copy_from_slice(body.as_slice()))
        });
        let mut multer = multer::Multipart::new(a, boundary.clone());
        let name = generate_string(32);
        if let Ok(mut file) = std::fs::File::create(&name) {
            while let Ok(Some(mut field)) = multer.next_field().await {
                if let Some(field_name) = field.name() {
                    if field_name.eq("file") {
                        if let Ok(mut info) = std::fs::File::create(format!("{}.json", name)) {
                            if let Some(name) = field.file_name() {
                                if let Err(error) =
                                    info.write_all(format!(r#"{{"name":"{}"}}"#, name).as_bytes())
                                {
                                    error!(error.to_string());
                                    return Err(FydiaResponse::new_error("File writing error"));
                                }
                            };

                            while let Ok(Some(chunck)) = field.chunk().await {
                                if let Err(error) = file.write_all(&chunck) {
                                    return Err(FydiaResponse::new_error(error.to_string()));
                                }
                            }
                        }
                    } else if field_name.eq("context") {
                        info!(field.text().await.unwrap_or_default());
                    }
                }
            }
        }

        let event = Event::new(
            server_id.clone(),
            EventContent::Message {
                content: Message::new(
                    name.clone(),
                    MessageType::FILE,
                    false,
                    SqlDate::new(DateTime::from(SystemTime::now())),
                    user.clone(),
                    channelid.clone(),
                ),
            },
        );
        Ok(event)
    } else {
        Err(FydiaResponse::new_error(""))
    }
}

pub fn json_message(
    body: String,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, FydiaResponse> {
    let message_type;
    let content;
    if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
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
    } else {
        return Err(FydiaResponse::new_error("Json error"));
    }
    if let Some(messagetype) = MessageType::from_string(message_type) {
        let event = Event::new(
            server_id.clone(),
            EventContent::Message {
                content: Message::new(
                    content,
                    messagetype,
                    false,
                    SqlDate::new(DateTime::from(SystemTime::now())),
                    user.clone(),
                    channelid.clone(),
                ),
            },
        );

        Ok(event)
    } else {
        Err(FydiaResponse::new_error("Bad Message Type"))
    }
}

pub fn get_mime_of_file(path: &str) -> Mime {
    let mut buf = [0; 16];

    if let Ok(mut file) = std::fs::File::open(&path) {
        if file.read(&mut buf).is_ok() {
            if let Some(e) = infer::get(&buf) {
                return mime::Mime::from_str(e.mime_type())
                    .unwrap_or(mime::APPLICATION_OCTET_STREAM);
            } else if let Ok(e) = std::fs::read_to_string(format!("{}.json", &path)) {
                if let Ok(value) = serde_json::from_str::<Value>(&e) {
                    if let Some(e) = value.get("name") {
                        if let Some(string) = e.as_str() {
                            return mime_guess::from_path(string).first_or_octet_stream();
                        }
                    }
                }
            }
        }
    };

    mime::APPLICATION_OCTET_STREAM
}
