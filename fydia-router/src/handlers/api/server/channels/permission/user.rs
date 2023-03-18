use fydia_sql::impls::permission::PermissionSql;

use fydia_sql::impls::user::SqlUser;

use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia};

use crate::handlers::basic::{ChannelFromId, Database, ServerJoinedFromId, UserFromToken};
use crate::handlers::{get_json, get_json_value_from_body};

/// Get permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn get_permission_of_user(
    UserFromToken(user): UserFromToken,
    ChannelFromId(channel): ChannelFromId,
    Database(database): Database,
) -> FydiaResult {
    let perm = Permission::of_user_in_channel(&channel.id, &user.id, &database).await?;

    FydiaResult::Ok(FydiaResponse::from_serialize(perm))
}

/// Post permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn post_permission_of_user(
    UserFromToken(user): UserFromToken,
    ChannelFromId(channel): ChannelFromId,
    ServerJoinedFromId(server): ServerJoinedFromId,
    Database(database): Database,
    body: String,
) -> FydiaResult {
    let perm = user.permission_of_server(&server.id, &database).await?;

    if !perm.can(&fydia_struct::permission::PermissionValue::Admin) {
        return FydiaResult::Err("Not enought permission".into_forbidden_error());
    }

    let json = get_json_value_from_body(&body)?;

    let value = get_json("value", &json)?
        .parse()
        .map_err(|_err| "Bad value")?;

    if let Ok(mut permission) =
        Permission::of_user_in_channel(&channel.id, &user.id, &database).await
    {
        permission.value = value;
        if permission.update_value(&database).await.is_err() {
            return FydiaResult::Err("Cannot update value".into_server_error());
        };
    } else {
        let perm = Permission {
            permission_type: fydia_struct::permission::PermissionType::User(user.id),
            channelid: Some(channel.id),
            value,
        };

        if perm.insert(&database).await.is_err() {
            return FydiaResult::Err("Cannot insert value".into_server_error());
        }
    }

    "".into()
}
