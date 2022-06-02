use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Return all server of user
///
/// # Errors
/// Return an error if the token isn't valid
pub async fn get_server_of_user<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let user = BasicValues::get_user(&headers, &database).await?;

    Ok(FydiaResponse::from_serialize(&user.servers))
}
