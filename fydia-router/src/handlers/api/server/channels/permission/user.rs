use axum::body::Bytes;
use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};

use crate::handlers::basic::BasicValues;

pub async fn get_permission_of_user<'a>(
    body: Bytes,
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, server, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = Permission::by_channel(&channel.id, &database)
        .await
        .map_err(|err| FydiaResponse::StringError(err))?;

    FydiaResult::Ok(FydiaResponse::Text("Default"))
}
