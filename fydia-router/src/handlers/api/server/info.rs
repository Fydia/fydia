use axum::extract::Extension;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn get_server_of_user(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let user = BasicValues::get_user(&headers, &database).await?;

    Ok(FydiaResponse::new_ok_json(&user.servers))
}
