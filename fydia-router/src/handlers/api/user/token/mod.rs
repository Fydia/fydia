use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::{handlers::basic::BasicValues, new_response};

pub async fn verify(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();

    if BasicValues::get_user(&headers, &database).await.is_ok() {
        FydiaResponse::new_ok("").update_response(&mut res);
    } else {
        FydiaResponse::new_error("").update_response(&mut res);
    }

    res
}
