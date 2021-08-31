use crate::handlers::api::websocket::Websockets;
use chrono::DateTime;
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
        "Bad Token".to_string(),
    );
    let headers = HeaderMap::borrow_from(&state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let extracted = ChannelExtractor::borrow_from(&state);
    let serverid = ServerId::new(extracted.serverid.clone());
    let channelid = extracted.channelid.clone();
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        return Ok((state, res));
    };

    if let Some(user) = token.get_user(database).await {
        if user.server.is_join(serverid.clone()) {
            let server = user
                .server
                .get(serverid.clone().short_id)
                .unwrap()
                .get_server(database)
                .await
                .unwrap();

            if server.channel.is_exists(channelid.clone()) {
                if headers.get("content-type").unwrap() == "application/json" {
                    let body = body::to_bytes(Body::take_from(&mut state))
                        .await
                        .expect("Error");

                    match String::from_utf8(body.to_vec()) {
                        Ok(string_body) => {
                            let value =
                                serde_json::from_str::<Value>(string_body.as_str()).unwrap();
                            match json_message(
                                value,
                                &user,
                                &ChannelId::new(channelid.clone()),
                                &serverid,
                            ) {
                                Ok(channel_msg) => {
                                    let mut websocket =
                                        Websockets::borrow_mut_from(&mut state).clone();
                                    let key = RsaData::borrow_from(&state).clone();
                                    let users = server.get_user(database).await;
                                    tokio::spawn(async move {
                                        websocket
                                            .send(
                                                &channel_msg.clone(),
                                                users.clone(),
                                                Some(&key),
                                                None,
                                            )
                                            .await;
                                    });
                                    *res.body_mut() = "".into();
                                    *res.status_mut() = StatusCode::OK;
                                }
                                Err(error) => {
                                    *res.status_mut() = StatusCode::BAD_REQUEST;
                                    *res.headers_mut().get_mut(CONTENT_TYPE).unwrap() =
                                        mime::APPLICATION_JSON.as_ref().parse().unwrap();
                                    *res.body_mut() =
                                        format! {r#"{{"status":"Error", "content":"{}"}}"#, error}
                                            .into();
                                }
                            }
                        }
                        Err(_) => {
                            *res.status_mut() = StatusCode::BAD_REQUEST;
                            *res.headers_mut().get_mut(CONTENT_TYPE).unwrap() =
                                mime::APPLICATION_JSON.as_ref().parse().unwrap();
                            *res.body_mut() =
                                format! {r#"{{"status":"Error", "content":"Utf-8 Error"}}"#}.into();
                        }
                    }
                } else if headers
                    .get("content-type")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains(&"multipart/form-data;")
                {
                    match multipart_message(
                        &mut state,
                        &user.clone(),
                        &ChannelId::new(channelid),
                        serverid,
                    )
                    .await
                    {
                        Ok(msg) => {
                            let mut websocket = Websockets::borrow_mut_from(&mut state).clone();
                            let key = RsaData::borrow_from(&state).clone();
                            let users = server.get_user(database).await;
                            tokio::spawn(async move {
                                websocket
                                    .send(&msg.clone(), users.clone(), Some(&key), None)
                                    .await;
                            });

                            *res.body_mut() = "Message send".into();
                            *res.status_mut() = StatusCode::OK;
                        }
                        Err(e) => {
                            *res.body_mut() = format!(r#"{{"error":"{}"}}"#, e).into();
                            *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        }
                    };
                } else {
                    *res.body_mut() = "Bad Content-Type".into();
                }
            } else {
                *res.body_mut() = "Unvalid channel".into();
            }
        } else {
            *res.body_mut() = "unknow server".into();
        }
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
    let boundary = headers
        .get(CONTENT_TYPE)
        .and_then(|ct| {
            let ct = ct.to_str().ok()?;
            let idx = ct.find(BOUNDARY)?;
            Some(ct[idx + BOUNDARY.len()..].to_string())
        })
        .unwrap();

    let mut multer = multer::Multipart::new(Body::take_from(state), boundary.clone());
    let name = generate_string(32);
    let mut file = std::fs::File::create(&name).unwrap();
    while let Ok(Some(mut field)) = multer.next_field().await {
        if field.name().unwrap().eq("file") {
            let mut info = std::fs::File::create(format!("{}.json", name)).unwrap();
            info.write_all(format!(r#"{{"name":"{}"}}"#, field.file_name().unwrap()).as_bytes())
                .unwrap();
            while let Ok(Some(chunck)) = field.chunk().await {
                match file.write_all(&chunck) {
                    Ok(_) => {}
                    Err(e) => return Err(e.to_string()),
                };
            }
            info!(format!("{:?}", get_mime_of_file(&name)));
        } else if let Some(e) = field.name() {
            if e.eq("context") {
                info!(field.text().await.unwrap_or_default());
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
}

pub fn json_message(
    value: Value,
    user: &User,
    channelid: &ChannelId,
    server_id: &ServerId,
) -> Result<Event, String> {
    let content = match value.get("content") {
        None => {
            return Err("Where is the content".to_string());
        }
        Some(content) => content.as_str().unwrap().to_string(),
    };
    let message_type = match value.get("type") {
        None => {
            return Err("Where is the type".to_string());
        }
        Some(msg_type) => msg_type.as_str().unwrap().to_string(),
    };
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
