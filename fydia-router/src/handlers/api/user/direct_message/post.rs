use std::sync::Arc;

use axum::extract::{BodyStream, Extension, Path};
use axum::response::IntoResponse;
use axum::{body::Body, http::Request};
use fydia_sql::impls::{channel::SqlDirectMessages, token::SqlToken, user::UserIdSql};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::{
    channel::DirectMessage,
    format::UserFormat,
    response::FydiaResponse,
    user::{Token, UserId},
};
use reqwest::StatusCode;

use crate::new_response;

pub async fn create_direct_message(
    request: Request<Body>,
    _body: BodyStream,
    Path(target_user): Path<String>,
    Extension(database): Extension<Arc<DbConnection>>,
) -> impl IntoResponse {
    let mut res = new_response();
    let headers = request.headers();
    if let Some(token) = Token::from_headervalue(headers) {
        if let Some(user) = token.get_user(&database).await {
            if let Some(user) = UserFormat::from_string(&target_user) {
                println!("{:?}", user);
                FydiaResponse::new_error_custom_status("Soon may be", StatusCode::NOT_IMPLEMENTED)
                    .update_response(&mut res);
            } else if let Ok(id) = target_user.parse::<i32>() {
                let target = UserId::new(id).get_user(&database).await;
                println!("{:?}", target);
                if let Some(target) = target {
                    let dm = DirectMessage::new(vec![UserId::new(user.id), UserId::new(target.id)]);
                    println!("{:?}", dm.insert(&database).await);
                } else {
                    FydiaResponse::new_error("Bad user id").update_response(&mut res);
                }
            } else {
                FydiaResponse::new_error("Bad user id").update_response(&mut res);
            }
        } else {
            FydiaResponse::new_error("Bad Token").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("No Token").update_response(&mut res);
    }
    info!(&target_user.to_string());
    res
}
