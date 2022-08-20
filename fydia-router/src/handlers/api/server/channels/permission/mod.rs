pub mod role;
pub mod user;

use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};

use crate::handlers::basic::BasicValues;

pub async fn get_permission<'a>(
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = Permission::by_channel(&channel.id, &database)
        .await
        .map_err(|err| FydiaResponse::StringError(err))?;

    FydiaResult::Ok(FydiaResponse::Json(
        fydia_utils::serde_json::to_value(perm).unwrap(),
    ))
}

pub async fn post_permission<'a>(
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perms = user
        .permission(&channel.id, &database)
        .await
        .unwrap()
        .calculate(channel.id);

    if !perms.can(&fydia_struct::permission::PermissionValue::Admin) {
        return FydiaResult::Err(FydiaResponse::TextError("Not enought permission"));
    }

    FydiaResult::Ok(FydiaResponse::Text("Permission added"))
}