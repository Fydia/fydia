use crate::handlers::basic::{ChannelFromId, Database, UserFromToken};
use fydia_sql::impls::{channel::SqlChannel, user::SqlUser};
use fydia_struct::response::{FydiaResponse, FydiaResult};

/// Return all message of channel
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn get_messages(
    UserFromToken(user): UserFromToken,
    ChannelFromId(channel): ChannelFromId,
    Database(database): Database,
) -> FydiaResult {
    if !user
        .permission_of_channel(&channel.id, &database)
        .await?
        .calculate(Some(channel.id.clone()))?
        .can_read()
    {
        return "Unknow channel".into();
    }

    let messages = channel.messages(&database).await?;

    FydiaResponse::from_serialize(messages).into()
}
