use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::{channel::SqlDirectMessages, user::UserIdSql};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::{
    channel::DirectMessage, format::UserFormat, response::FydiaResponse, user::UserId,
};
use http::HeaderMap;
use reqwest::StatusCode;

use crate::handlers::basic::BasicValues;
use crate::new_response;

pub async fn create_direct_message(
    headers: HeaderMap,
    Path(target_user): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    let user = match BasicValues::get_user(&headers, &database).await {
        Ok(v) => v,
        Err(error) => {
            FydiaResponse::new_error(error).update_response(&mut res);
            return res;
        }
    };

    if let Some(user) = UserFormat::from_string(&target_user) {
        println!("{:?}", user);
        FydiaResponse::new_error_custom_status("Soon may be", StatusCode::NOT_IMPLEMENTED)
            .update_response(&mut res);
    } else if let Ok(id) = target_user.parse::<i32>() {
        let target = UserId::new(id).get_user(&database).await;
        println!("{:?}", target);
        if let Some(target) = target {
            let dm = DirectMessage::new(vec![user.id, target.id]);
            println!("{:?}", dm.insert(&database).await);
        } else {
            FydiaResponse::new_error("Bad user id").update_response(&mut res);
        }
    } else {
        FydiaResponse::new_error("Bad user id").update_response(&mut res);
    }

    info!(&target_user.to_string());

    res
}
