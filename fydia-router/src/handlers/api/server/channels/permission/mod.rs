pub mod role;
pub mod user;

use crate::handlers::basic::{ChannelFromId, Database};
use fydia_sql::impls::permission::PermissionSql;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};

/// Get permission
///
/// # Errors
/// Return an error if :
/// * channelid, serverid isn't valid
pub async fn get_permission(
    ChannelFromId(channel): ChannelFromId,
    Database(database): Database,
) -> FydiaResult {
    let perm = Permission::of_channel(&channel.id, &database).await?;

    FydiaResponse::from_serialize(perm).into()
}
