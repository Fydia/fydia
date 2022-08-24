use std::num::ParseIntError;

use axum::body::Bytes;
use axum::http::StatusCode;
use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::role::SqlRoles;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::roles::{Role, RoleId};

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};

/// Get permission of role
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn get_permission_of_role<'a>(
    Path((serverid, channelid, roleid)): Path<(String, String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (_, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let roleid = roleid
        .as_str()
        .parse()
        .map_err(|err: ParseIntError| FydiaResponse::StringError(err.to_string()))?;

    let role = Role::by_id(roleid, &server.id, &database)
        .await
        .map_err(FydiaResponse::StringError)?;

    let perm = Permission::of_role_in_channel(&channel.id, &role.id, &database)
        .await
        .map_err(FydiaResponse::StringError)?;

    FydiaResult::Ok(FydiaResponse::from_serialize(perm))
}

/// Post permission of user
///
/// # Errors
/// Return an error if :
/// * channelid, serverid, roleid isn't valid
pub async fn post_permission_of_user<'a>(
    body: Bytes,
    Path((serverid, channelid, roleid)): Path<(String, String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = user
        .permission_of_server(&server.id, &database)
        .await
        .map_err(FydiaResponse::StringError)?;

    if !perm.can(&fydia_struct::permission::PermissionValue::Admin) {
        return FydiaResult::Err(FydiaResponse::TextErrorWithStatusCode(
            StatusCode::FORBIDDEN,
            "Not enought permission",
        ));
    }

    let json = get_json_value_from_body(&body).map_err(FydiaResponse::StringError)?;

    let value = get_json("value", &json)?.parse().map_err(|_err| {
        FydiaResponse::TextErrorWithStatusCode(StatusCode::INTERNAL_SERVER_ERROR, "Bad value")
    })?;

    let roleid = RoleId::Id(
        roleid
            .parse()
            .map_err(|_err| FydiaResponse::TextError("Bad Value"))?,
    );

    if let Ok(mut permission) =
        Permission::of_role_in_channel(&channel.id, &roleid, &database).await
    {
        permission.value = value;
        if permission.update_value(&database).await.is_err() {
            return FydiaResult::Err(FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot update value",
            ));
        };
    } else {
        let perm = Permission {
            permission_type: fydia_struct::permission::PermissionType::Role(roleid),
            channelid: Some(channel.id),
            value,
        };

        if perm.insert(&database).await.is_err() {
            return FydiaResult::Err(FydiaResponse::TextErrorWithStatusCode(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot insert value",
            ));
        }
    }

    FydiaResult::Ok(FydiaResponse::Text(""))
}
