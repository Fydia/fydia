use axum::body::Bytes;
use axum::extract::{Extension, Path};
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelType, ParentId};
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;
use crate::handlers::get_json;

pub async fn create_channel(
    body: Bytes,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, serverid, &database).await?;

    let body = String::from_utf8(body.to_vec())
        .map_err(|_| FydiaResponse::new_error("Body isn't UTF-8"))?;

    let json = serde_json::from_str::<Value>(body.as_str())
        .map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let name = get_json("name", &json)?.to_string();
    let channeltype = ChannelType::from_string(get_json("type", &json)?.to_string());

    let channel = Channel::new_with_parentid(
        name,
        "".to_string(),
        ParentId::ServerId(server.id.clone()),
        channeltype,
    )
    .map_err(FydiaResponse::new_error)?;

    server
        .insert_channel(&channel, &database)
        .await
        .map(|_| FydiaResponse::new_ok(channel.id.id))
        .map_err(|_| FydiaResponse::new_error("Cannot create the channel"))
}
