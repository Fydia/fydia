use crate::handlers::basic::{
    ChannelFromId, Database, RoleFromId, ServerJoinedFromId, UserFromToken,
};
use crate::handlers::{get_json, get_json_value_from_body};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::user::SqlUser;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia};

/// Get permission of role
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn get_permission_of_role(
    ServerJoinedFromId(_): ServerJoinedFromId,
    ChannelFromId(channel): ChannelFromId,
    RoleFromId(role): RoleFromId,
    Database(database): Database,
) -> FydiaResult {
    let perm = Permission::of_role_in_channel(&channel.id, &role.id, &database).await?;

    FydiaResult::Ok(FydiaResponse::from_serialize(perm))
}

/// Post permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn post_permission_of_user(
    UserFromToken(user): UserFromToken,
    ServerJoinedFromId(server): ServerJoinedFromId,
    ChannelFromId(channel): ChannelFromId,
    RoleFromId(role): RoleFromId,
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
        .map_err(|_err| "Bad value".into_server_error())?;

    if let Ok(mut permission) =
        Permission::of_role_in_channel(&channel.id, &role.id, &database).await
    {
        permission.value = value;
        if permission.update_value(&database).await.is_err() {
            return FydiaResult::Err("Cannot update value".into_server_error());
        };
    } else {
        let perm = Permission {
            permission_type: fydia_struct::permission::PermissionType::Role(role.id),
            channelid: Some(channel.id),
            value,
        };

        if perm.insert(&database).await.is_err() {
            return FydiaResult::Err("Cannot insert value".into_server_error());
        }
    }

    Ok("".into_ok())
}
