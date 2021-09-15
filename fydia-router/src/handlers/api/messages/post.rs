use crate::handlers::api::websocket::Websockets;
use chrono::DateTime;
use fydia_sql::impls::message::SqlMessage;
use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::instance::RsaData;
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::pathextractor::ChannelExtractor;
use fydia_struct::server::ServerId;
use fydia_struct::user::{Token, User};
use fydia_utils::generate_string;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::header::CONTENT_TYPE;
use gotham::hyper::{body, Body, HeaderMap, StatusCode};
use gotham::state::{FromState, State};
use mime::Mime;
use serde_json::Value;
use std::io::{Read, Write};
use std::str::FromStr;
use std::time::SystemTime;

const BOUNDARY: &str = "boundary=";

pub async fn post_messages(mut state: State) -> HandlerResult {
    let mut res = create_response(
        &state,
        StatusCode::BAD_REQUEST,
        mime::TEXT_PLAIN_UTF_8,
        "Error".to_string(),
    );
    let headers = HeaderMap::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let extracted = ChannelExtractor::borrow_from(&state);
    let serverid = ServerId::new(extracted.serverid.clone());
    let channelid = extracted.channelid.clone();
    let token = if let Some(token) = Token::from_headervalue(headers) {
        token
    } else {
        *res.body_mut() = "Bad Token".into();
        return Ok((state, res));
    };

    if let Some(user) = token.get_user(database).await {
        if user.server.is_join(serverid.clone()) {
            if let Some(serverid) = user.server.get(serverid.clone().short_id) {
                if let Ok(server) = serverid.get_server(database).await {
                    if server.channel.is_exists(channelid.clone()) {
                        let msg = if headers.get("content-type").unwrap() == "application/json" {
                            if let Ok(body) = body::to_bytes(Body::take_from(&mut state)).await {
                                match String::from_utf8(body.to_vec()) {
                                    Ok(string_body) => {
                                        if let Ok(value) =
                                            serde_json::from_str::<Value>(string_body.as_str())
                                        {
                                            match json_message(
                                                value,
                                                &user,
                                                &ChannelId::new(channelid.clone()),
                                                &serverid,
                                            ) {
                                                Ok(msg) => msg,
                                                Err(error) => {
                                                    if let Some(header) =
                                                        res.headers_mut().get_mut(CONTENT_TYPE)
                                                    {
                                                        if let Ok(content_type) =
                                                            mime::APPLICATION_JSON.as_ref().parse()
                                                        {
                                                            *header = content_type;
                                                        }
                                                    }
                                                    *res.status_mut() = StatusCode::BAD_REQUEST;
                                                    *res.body_mut() =
                                                        format! {r#"{{"status":"Error", "content":"{}"}}"#, error}
                                                            .into();

                                                    return Ok((state, res));
                                                }
                                            }
                                        } else {
                                            *res.status_mut() = StatusCode::BAD_REQUEST;
                                            *res.body_mut() =
                                                r#"{{"status":"Error", "content":"Bad Json"}}"#
                                                    .into();
                                            return Ok((state, res));
                                        }
                                    }
                                    Err(_) => {
                                        *res.status_mut() = StatusCode::BAD_REQUEST;
                                        if let Some(header) =
                                            res.headers_mut().get_mut(CONTENT_TYPE)
                                        {
                                            if let Ok(content_type) =
                                                mime::APPLICATION_JSON.as_ref().parse()
                                            {
                                                *header = content_type;
                                            }
                                        }
                                        *res.body_mut() =
                                            r#"{{"status":"Error", "content":"Utf-8 Error"}}"#
                                                .into();
                                        return Ok((state, res));
                                    }
                                }
                            } else {
                                *res.status_mut() = StatusCode::BAD_REQUEST;
                                *res.body_mut() =
                                    r#"{{"status":"Error", "content":"Utf-8 Error"}}"#.into();
                                if let Some(header) = res.headers_mut().get_mut(CONTENT_TYPE) {
                                    if let Ok(content_type) =
                                        mime::APPLICATION_JSON.as_ref().parse()
                                    {
                                        *header = content_type;
                                    }
                                }

                                return Ok((state, res));
                            }
                        } else if let Some(header) = headers.get(CONTENT_TYPE) {
                            if let Ok(string) = header.to_str() {
                                if string.contains(&"multipart/form-data;") {
                                    match multipart_message(
                                        &mut state,
                                        &user.clone(),
                                        &ChannelId::new(channelid),
                                        serverid,
                                    )
                                    .await
                                    {
                                        Ok(msg) => msg,
                                        Err(e) => {
                                            *res.body_mut() =
                                                format!(r#"{{"error":"{}"}}"#, e).into();
                                            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                            return Ok((state, res));
                                        }
                                    }
                                } else {
                                    return Ok((state, res));
                                }
                            } else {
                                *res.body_mut() =
                                    r#"{{"status":"Error", "content":"Utf-8 Error"}}"#.into();
                                return Ok((state, res));
                            }
                        } else {
                            *res.body_mut() = "Bad Content-Type".into();
                            return Ok((state, res));
                        };
                        let mut websocket = Websockets::borrow_mut_from(&mut state).clone();
                        let key = RsaData::borrow_from(&state).clone();
                        if let Ok(members) = server.get_user(database).await {
                            if let EventContent::Message { ref content } = msg.content {
                                if content.insert_message(database).await.is_err() {
                                    *res.body_mut() = "Cannot send message".into();
                                    *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                } else {
                                    tokio::spawn(async move {
                                        websocket
                                            .send(
                                                &msg.clone(),
                                                members.members.clone(),
                                                Some(&key),
                                                None,
                                            )
                                            .await;
                                    });
                                }
                            }

                            *res.body_mut() = "Message send".into();
                            *res.status_mut() = StatusCode::OK;
                        } else {
                            *res.body_mut() = "Cannot get users of the server".into();
                            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    } else {
                        *res.body_mut() = "Unvalid channel".into();
                    }
                } else {
                    *res.body_mut() = "unknow server".into();
                }
            } else {
                *res.body_mut() = "unknow server".into();
            }
        } else {
            *res.body_mut() = "unknow server".into();
        }
    } else {
        *res.body_mut() = "Token error".into();
    }

    Ok((state, res))
}

pub async fn multipart_message(
    state: &mut State,
    user: &User,
    channelid: &ChannelId,
    server_id: ServerId,
) -> Result<Event, String> {
    let headers = HeaderMap::borrow_from(state);

    if let Some(boundary) = headers.get(CONTENT_TYPE).and_then(|ct| {
        let ct = ct.to_str().ok()?;
        let idx = ct.find(BOUNDARY)?;
        Some(ct[idx + BOUNDARY.len()..].to_string())
    }) {
        let mut multer = multer::Multipart::new(Body::take_from(state), boundary.clone());
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
                                    return Err(String::from("File writing error"));
                                }
                            };

                            while let Ok(Some(chunck)) = field.chunk().await {
                                if let Err(e) = file.write_all(&chunck) {
                                    return Err(e.to_string());
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
            server_id,
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
        Err(String::from("error"))
    }
}

pub fn json_message(
    value: Value,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, String> {
    let message_type;
    let content;
    match (value.get("type"), value.get("content")) {
        (Some(ctype), Some(mcontent)) => match (ctype.as_str(), mcontent.as_str()) {
            (Some(ctype), Some(mcontent)) => {
                message_type = ctype.to_string();
                content = mcontent.to_string();
            }
            _ => {
                return Err("Json error".to_string());
            }
        },
        _ => {
            return Err("Json error".to_string());
        }
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
        Err(String::from("Bad Message Type"))
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
