use crate::handlers::basic::{ChannelFromId, Database};

use fydia_sql::impls::channel::SqlChannel;
use fydia_struct::response::FydiaResult;

/// Delete a channel in a server
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn delete_channel(
    Database(database): Database,
    ChannelFromId(channel): ChannelFromId,
) -> FydiaResult {
    channel.delete(&database).await?;

    "Channel deleted".into()
}
