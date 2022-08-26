use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResult, IntoFydia};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Return a 200 OK if token is valid
///
/// # Errors
/// This function will return an error if token isn't valid
pub async fn verify<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    if let Err(res) = BasicValues::get_user(&headers, &database).await {
        return Err(res);
    }

    Ok("".into_ok())
}
