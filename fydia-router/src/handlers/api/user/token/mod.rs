use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::{impls::user::SqlUser, sqlpool::DbConnection};
use fydia_struct::{
    response::FydiaResponse,
    user::{Token, User},
};
use http::HeaderMap;

use crate::new_response;

pub async fn verify(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    let token = if let Some(token) = Token::from_headervalue(&headers) {
        token
    } else {
        FydiaResponse::new_error("").update_response(&mut res);

        return res;
    };

    if User::get_user_by_token(&token, &database).await.is_some() {
        FydiaResponse::new_ok("").update_response(&mut res);
    } else {
        FydiaResponse::new_error("").update_response(&mut res);
    }

    res
}
