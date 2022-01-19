use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::new_response;

pub async fn get_server_of_user(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    match BasicValues::get_user(&headers, &database).await {
        Ok(user) => {
            FydiaResponse::new_ok_json(&user.servers).update_response(&mut res);
        }
        Err(error) => {
            FydiaResponse::new_error(error).update_response(&mut res);
        }
    }

    res
}
