pub mod role;
pub mod user;

use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};

use crate::handlers::basic::BasicValues;

pub async fn get_permission<'a>(
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (_, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = Permission::by_channel(&channel.id, &database)
        .await
        .map_err(|err| FydiaResponse::StringError(err))?;

    println!("{:?}", perm);

    if !perm.can(&fydia_struct::permission::PermissionValue::Read) {
        return Ok(FydiaResponse::TextError("Unknow channel"));
    }

    FydiaResult::Ok(FydiaResponse::Text("Default"))
}
