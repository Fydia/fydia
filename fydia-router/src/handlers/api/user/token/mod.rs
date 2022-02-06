use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn verify(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    BasicValues::get_user(&headers, &database)
        .await
        .map(|_| FydiaResponse::new_ok(""))
        .map_err(|_| FydiaResponse::new_error(""))
}
