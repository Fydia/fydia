use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::user::{Token, User};
use http::HeaderMap;

use crate::new_response;

pub async fn get_server_of_user(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        FydiaResponse::new_error("Bad Token").update_response(&mut res);

        return res;
    };

    if let Some(user) = User::get_user_by_token(&token, &database).await {
        FydiaResponse::new_ok_json(&user.server).update_response(&mut res);
    } else {
        FydiaResponse::new_error("Token error").update_response(&mut res);
    }

    res
}
