use axum::extract::{Path, State};
use fydia_sql::impls::server::SqlServer;
use fydia_struct::channel::{Channel, ChannelType};
use fydia_struct::response::{FydiaResult, IntoFydia, MapError};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};
use crate::ServerState;

/// Create a new channel in a server
///
/// # Errors
/// Return an error if:
/// * serverid, token isn't valid
/// * body isn't valid
/// * database is unreachable
pub async fn create_channel(
    Path(serverid): Path<String>,
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: String,
) -> FydiaResult {
    let (_, mut server) =
        BasicValues::get_user_and_server_and_check_if_joined(&headers, &serverid, &state.database)
            .await?;

    let json = get_json_value_from_body(&body)?;

    let name = get_json("name", &json)?.to_string();
    let channeltype = ChannelType::from_string(get_json("type", &json)?.to_string());

    let channel = Channel::new_with_serverid(name, "".to_string(), server.id.clone(), channeltype)
        .error_to_fydiaresponse()?;

    server
        .insert_channel(&channel, &state.database)
        .await
        .map(|_| channel.id.id.into_ok())
}
