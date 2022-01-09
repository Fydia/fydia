use axum::{extract::Extension, response::IntoResponse};
use fydia_sql::{impls::token::SqlToken, sqlpool::DbConnection};
use fydia_struct::{user::Token, response::FydiaResponse};
use http::HeaderMap;

use crate::new_response;

pub async fn get_info_of_self(headers: HeaderMap, Extension(executor): Extension<DbConnection>) -> impl IntoResponse {
    let mut res = new_response();
    if let Some(token) = Token::from_headervalue(&headers) {
        if let Some(user) = token.get_user(&executor).await {
            if let Ok(json) = serde_json::to_string(&user.to_userinfo()) {
                FydiaResponse::new_ok_json(json).update_response(&mut res);
            }
        } else {
            FydiaResponse::new_error("This token is wrong").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("No Token").update_response(&mut res);
    }
    return res;
}
