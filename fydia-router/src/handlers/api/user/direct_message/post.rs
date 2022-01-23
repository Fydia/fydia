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

pub async fn create_direct_message(
    headers: HeaderMap,
    Path(target_user): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let user = match BasicValues::get_user(&headers, &database).await {
        Ok(v) => v,
        Err(error) => {
            return error;
        }
    };

    if let Some(user) = UserFormat::from_string(&target_user) {
        println!("{:?}", user);
        return FydiaResponse::new_error_custom_status("Soon may be", StatusCode::NOT_IMPLEMENTED);
    } else if let Ok(id) = target_user.parse::<i32>() {
        let target = UserId::new(id).get_user(&database).await;
        println!("{:?}", target);
        if let Some(target) = target {
            let dm = DirectMessage::new(vec![user.id, target.id]);
            println!("{:?}", dm.insert(&database).await);
            return FydiaResponse::new_ok("");
        }
    }

    info!(&target_user.to_string());

    return FydiaResponse::new_error("Bad user id");
}
