use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::HeaderMap;

pub mod channels;
pub mod create;
pub mod info;
pub mod join;
pub mod picture;
pub mod roles;

/// Return requested server
///
/// # Errors
/// This function will return if the token or serverid isn't valid
pub async fn get_server<'a>(
    headers: HeaderMap,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    BasicValues::get_user_and_server_and_check_if_joined(&headers, &serverid, &database)
        .await
        .map(|(_, server)| FydiaResponse::from_serialize(&server))
}
