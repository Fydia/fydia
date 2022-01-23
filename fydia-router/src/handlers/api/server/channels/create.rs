use axum::body::Bytes;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelType, ParentId};
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;

pub async fn create_channel(
    body: Bytes,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_, mut server) =
        match BasicValues::get_user_and_server_and_check_if_joined(&headers, serverid, &database)
            .await
        {
            Ok(v) => v,
            Err(error) => {
                return FydiaResponse::new_error(error);
            }
        };

    if let Ok(body) = String::from_utf8(body.to_vec()) {
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
                            ChannelType::from_string(ctype),
                        );
                        if let Err(error) = server.insert_channel(channel.clone(), &database).await
                        {
                            error!(error);
                            return FydiaResponse::new_error("Cannot create the channel");
                        } else {
                            return FydiaResponse::new_ok(channel.id.id);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            return FydiaResponse::new_error("Error with name or Channel Type");
        }

        return FydiaResponse::new_error("Json error");
    }

    FydiaResponse::new_error("Body isn't UTF-8")
}
