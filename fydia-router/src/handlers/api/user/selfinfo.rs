use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn get_info_of_self(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
) -> impl IntoResponse {
    match BasicValues::get_user(&headers, &executor).await {
        Ok(user) => FydiaResponse::new_ok_json(&user.to_userinfo()),
        Err(error) => error,
    }
}
