use axum::extract::{self, Extension, Path};
use axum::response::IntoResponse;
use futures::StreamExt;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelType};
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::{Token, User};
use http::HeaderMap;
use serde_json::Value;

use crate::new_response;

pub async fn create_channel(
    mut body: extract::BodyStream,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let mut res = new_response();

    let token = Token::from_headervalue(&headers);

    if let Some(token) = token {
        if let Some(user) = User::get_user_by_token(&token, &database).await {
            if user.servers.is_join(ServerId::new(serverid.clone())) {
                while let Some(Ok(vec)) = body.next().await {
                    if let Ok(body) = String::from_utf8(vec.to_vec()) {
                        if let Ok(mut server) =
                            Server::get_server_by_id(ServerId::new(serverid.to_string()), &database)
                                .await
                        {
                            let mut channel = Channel::new();
                            channel.parent_id = fydia_struct::channel::ParentId::ServerId(
                                ServerId::new(server.id.clone()),
                            );

                            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                                let name = value.get("name");
                                let ctype = value.get("type");

                                match (name, ctype) {
                                    (Some(name), Some(ctype)) => {
                                        match (name.as_str(), ctype.as_str()) {
                                            (Some(name), Some(ctype)) => {
                                                channel.name = name.to_string();
                                                channel.channel_type =
                                                    ChannelType::from_string(ctype.to_string());

                                                if let Err(error) = server
                                                    .insert_channel(channel.clone(), &database)
                                                    .await
                                                {
                                                    FydiaResponse::new_error(
                                                        "Cannot create the channel",
                                                    )
                                                    .update_response(&mut res);
                                                    error!(error);
                                                } else {
                                                    FydiaResponse::new_ok(channel.id)
                                                        .update_response(&mut res);
                                                }
                                            }
                                            _ => FydiaResponse::new_error(
                                                "Error with name or Channel Type",
                                            )
                                            .update_response(&mut res),
                                        }
                                    }
                                    _ => {
                                        FydiaResponse::new_error("Error with name or Channel Type")
                                            .update_response(&mut res);
                                    }
                                }
                            }
                        } else {
                            FydiaResponse::new_error("Cannot get server").update_response(&mut res)
                        }
                    } else {
                        FydiaResponse::new_error("Body isn't UTF-8").update_response(&mut res)
                    }
                }
            } else {
                FydiaResponse::new_error("Unknow Server").update_response(&mut res)
            }
        } else {
            FydiaResponse::new_error("User error").update_response(&mut res)
        }
    } else {
        FydiaResponse::new_error("Token error").update_response(&mut res)
    }

    res
}
