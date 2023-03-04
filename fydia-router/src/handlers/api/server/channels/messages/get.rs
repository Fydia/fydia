use crate::handlers::basic::{ChannelFromId, Database, UserFromToken};
use fydia_sql::impls::{channel::SqlChannel, user::SqlUser};
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia, MapError};

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
        .calculate(Some(channel.id.clone()))
        .error_to_fydiaresponse()?
        .can_read()
    {
        return FydiaResult::Err("Unknow channel".into_error());
    }

    channel
        .messages(&database)
        .await
        .map(FydiaResponse::from_serialize)
}
