use std::convert::TryInto;

use axum::body::Bytes;
use axum::http::StatusCode;
use axum::Extension;
use axum::{extract::Path, http::HeaderMap};
use fydia_sql::impls::permission::PermissionSql;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::permission::Permission;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::user::UserId;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};

pub async fn get_permission_of_user<'a>(
    Path((serverid, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
    headers: HeaderMap,
) -> FydiaResult<'a> {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    let perm = Permission::of_user_in_channel(&channel.id, &user.id, &database)
        .await
        .map_err(|err| FydiaResponse::StringError(err))?;

    FydiaResult::Ok(FydiaResponse::Json(
        fydia_utils::serde_json::to_value(perm).unwrap(),
    ))
}

pub async fn post_permission_of_user<'a>(
    body: Bytes,
    Path((serverid, channelid, userid)): Path<(String, String, String)>,
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
        .map_err(|err| FydiaResponse::StringError(err))?;

    if !perm.can(&fydia_struct::permission::PermissionValue::Admin) {
        return FydiaResult::Err(FydiaResponse::TextErrorWithStatusCode(
            StatusCode::FORBIDDEN,
            "Not enought permission",
        ));
    }

    let json = get_json_value_from_body(&body).map_err(FydiaResponse::StringError)?;

    let value = get_json("value", &json)?.parse().map_err(|_| {
        FydiaResponse::TextErrorWithStatusCode(StatusCode::INTERNAL_SERVER_ERROR, "Bad value")
    })?;

    let userid = UserId::new(userid.parse().unwrap());

    if let Ok(mut permission) =
        Permission::of_user_in_channel(&channel.id, &userid, &database).await
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
            permission_type: fydia_struct::permission::PermissionType::User(userid),
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
