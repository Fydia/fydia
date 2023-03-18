use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::impls::user::SqlUser;

use fydia_struct::response::FydiaResult;

use crate::handlers::basic::{ChannelFromId, Database, UserFromToken};
use crate::handlers::{get_json, get_json_value_from_body};

/// Change name of a channel
///
/// # Errors
/// Return an error if serverid or channelid or body isn't valid
pub async fn update_name(
    UserFromToken(user): UserFromToken,
    ChannelFromId(mut channel): ChannelFromId,
    Database(database): Database,
    body: String,
) -> FydiaResult {
    if !user
        .permission_of_channel(&channel.id, &database)
        .await?
        .calculate(Some(channel.id.clone()))?
        .can_read()
    {
        return "Unknow channel".into();
    }

    let json = get_json_value_from_body(&body)?;

    let name = get_json("name", &json)?;

    channel.update_name(name, &database).await?;

    "Channel name updated".into()
}

/// Change description of a channel
///
/// # Errors
/// Return an error if channelid or serverid or body isn't valid
pub async fn update_description(
    ChannelFromId(mut channel): ChannelFromId,
    Database(database): Database,
    body: String,
) -> FydiaResult {
    let json = get_json_value_from_body(&body)?;

    let description = get_json("description", &json)?;

    channel.update_description(description, &database).await?;

    "Channel description updated".into()
}
