use axum::extract::State;
use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia};
use fydia_struct::user::UserId;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};
use crate::ServerState;

/// Get permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn get_permission_of_user(
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = Permission::of_user_in_channel(&channel.id, &user.id, &database).await?;

    FydiaResult::Ok(FydiaResponse::from_serialize(perm))
}

/// Post permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn post_permission_of_user(
    Path((serverid, channelid, userid)): Path<(String, String, String)>,
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: String,
) -> FydiaResult {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers,
        &serverid,
        &channelid,
        &state.database,
    )
    .await?;

    let perm = user
        .permission_of_server(&server.id, &state.database)
        .await?;

    if !perm.can(&fydia_struct::permission::PermissionValue::Admin) {
        return FydiaResult::Err("Not enought permission".into_forbidden_error());
    }

    let json = get_json_value_from_body(&body)?;

    let value = get_json("value", &json)?
        .parse()
        .map_err(|_err| "Bad value".into_error())?;

    let userid = UserId::new(userid.parse().map_err(|_err| "Bad Value".into_error())?);

    if let Ok(mut permission) =
        Permission::of_user_in_channel(&channel.id, &userid, &state.database).await
    {
        permission.value = value;
        if permission.update_value(&state.database).await.is_err() {
            return FydiaResult::Err("Cannot update value".into_server_error());
        };
    } else {
        let perm = Permission {
            permission_type: fydia_struct::permission::PermissionType::User(userid),
            channelid: Some(channel.id),
            value,
        };

        if perm.insert(&state.database).await.is_err() {
            return FydiaResult::Err("Cannot insert value".into_server_error());
        }
    }

    Ok("".into_ok())
}
