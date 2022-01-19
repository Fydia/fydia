use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::{handlers::basic::BasicValues, new_response};

pub async fn get_info_of_self(
    headers: HeaderMap,
    Extension(executor): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    match BasicValues::get_user(&headers, &executor).await {
        Ok(user) => {
            FydiaResponse::new_ok_json(&user.to_userinfo()).update_response(&mut res);
        }
        Err(error) => {
            FydiaResponse::new_error(error).update_response(&mut res);
        }
    }

    res
}
