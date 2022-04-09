use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn verify<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    BasicValues::get_user(&headers, &database)
        .await
        .map(|_| FydiaResponse::Text(""))
        .map_err(|_| FydiaResponse::TextError(""))
}
