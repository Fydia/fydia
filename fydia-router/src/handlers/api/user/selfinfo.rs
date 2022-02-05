use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn get_info_of_self(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
) -> FydiaResult {
    BasicValues::get_user(&headers, &executor)
        .await
        .map(|user| FydiaResponse::new_ok_json(&user.to_userinfo()))
}
