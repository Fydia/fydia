use axum::body::Bytes;
use axum::extract::{Extension, Path};
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelType};
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};

/// Create a new channel in a server
///
/// # Errors
/// Return an error if:
/// * serverid, token isn't valid
/// * body isn't valid
/// * database is unreachable
pub async fn create_channel<'a>(
    body: Bytes,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, &serverid, &database)
            .await?;

    let json = get_json_value_from_body(&body)?;

    let name = get_json("name", &json)?.to_string();
    let channeltype = ChannelType::from_string(get_json("type", &json)?.to_string());

    let channel = Channel::new_with_serverid(name, "".to_string(), server.id.clone(), channeltype)
        .map_err(FydiaResponse::StringError)?;

    server
        .insert_channel(&channel, &database)
        .await
        .map(|_| FydiaResponse::String(channel.id.id))
}
