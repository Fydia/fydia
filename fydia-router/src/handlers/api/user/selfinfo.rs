use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult, MapError};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Get info of user
///
/// # Errors
/// This function will return an error if the token is wrong
pub async fn get_info_of_self<'a>(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let value = BasicValues::get_user(&headers, &executor)
        .await?
        .self_json_output()
        .error_to_fydiaresponse()?;

    Ok(FydiaResponse::from_serialize(value))
}
