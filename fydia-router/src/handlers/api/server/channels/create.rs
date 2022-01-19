use axum::extract::{self, Extension, Path};
use axum::response::IntoResponse;
use futures::StreamExt;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelType, ParentId};
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;
use crate::new_response;

pub async fn create_channel(
    mut body: extract::BodyStream,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let mut res = new_response();

    let (_, mut server) =
        match BasicValues::get_user_and_server_and_check_if_joined(&headers, serverid, &database)
            .await
        {
            Ok(v) => v,
            Err(error) => {
                FydiaResponse::new_error(error).update_response(&mut res);
                return res;
            }
        };

    while let Some(Ok(vec)) = body.next().await {
        if let Ok(body) = String::from_utf8(vec.to_vec()) {
            if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                let name = value.get("name");
                let ctype = value.get("type");

                match (name, ctype) {
                    (Some(name), Some(ctype)) => match (name.as_str(), ctype.as_str()) {
                        (Some(name), Some(ctype)) => {
                            let channel = Channel::new_with_parentid(
                                name,
                                "",
                                ParentId::ServerId(server.id.clone()),
                                ChannelType::from_string(ctype.to_string()),
                            );
                            if let Err(error) =
                                server.insert_channel(channel.clone(), &database).await
                            {
                                FydiaResponse::new_error("Cannot create the channel")
                                    .update_response(&mut res);
                                error!(error);
                            } else {
                                FydiaResponse::new_ok(channel.id.id).update_response(&mut res);
                            }
                        }
                        _ => FydiaResponse::new_error("Error with name or Channel Type")
                            .update_response(&mut res),
                    },
                    _ => {
                        FydiaResponse::new_error("Error with name or Channel Type")
                            .update_response(&mut res);
                    }
                }
            }
        } else {
            FydiaResponse::new_error("Body isn't UTF-8").update_response(&mut res)
        }
    }

    res
}
