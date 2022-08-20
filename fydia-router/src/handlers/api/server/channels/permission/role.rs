use std::num::ParseIntError;

use axum::body::Bytes;
use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::role::SqlRoles;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::roles::Role;

use crate::handlers::basic::BasicValues;

pub async fn get_permission_of_role<'a>(
    body: Bytes,
    Path((serverid, channelid, roleid)): Path<(String, String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;
    let roleid = roleid
        .as_str()
        .parse()
        .map_err(|err: ParseIntError| FydiaResponse::StringError(err.to_string()))?;

    let role = Role::by_id(roleid, &server.id, &database).await.unwrap();

    let perm = Permission::by_role(&channel.id, &role, &database)
        .await
        .map_err(|err| FydiaResponse::StringError(err))?;

    FydiaResult::Ok(FydiaResponse::Json(
        fydia_utils::serde_json::to_value(perm).unwrap(),
    ))
}
