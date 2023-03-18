use fydia_sql::impls::server::SqlServer;
use fydia_struct::channel::{Channel, ChannelType};
use fydia_struct::response::FydiaResult;

use crate::handlers::basic::{Database, ServerJoinedFromId};
use crate::handlers::{get_json, get_json_value_from_body};

/// Create a new channel in a server
///
/// # Errors
/// Return an error if:
/// * serverid, token isn't valid
/// * body isn't valid
/// * database is unreachable
pub async fn create_channel(
    ServerJoinedFromId(mut server): ServerJoinedFromId,
    Database(database): Database,
    body: String,
) -> FydiaResult {
    let json = get_json_value_from_body(&body)?;

    let name = get_json("name", &json)?.to_string();
    let channeltype = ChannelType::from_string(get_json("type", &json)?.to_string());

    let channel = Channel::new_with_serverid(name, "".to_string(), server.id.clone(), channeltype)?;

    server
        .insert_channel(&channel, &database)
        .await
        .map(|_| channel.id.id)
        .into()
}
