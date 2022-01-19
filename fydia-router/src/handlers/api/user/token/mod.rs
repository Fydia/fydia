use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn verify(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    if BasicValues::get_user(&headers, &database).await.is_ok() {
        FydiaResponse::new_ok("")
    } else {
        FydiaResponse::new_error("")
    }
}
