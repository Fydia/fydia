use axum::extract::{Extension, Path};
use fydia_sql::impls::{channel::SqlDirectMessages, user::UserFrom};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResult;
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
) -> FydiaResult {
    let user = BasicValues::get_user(&headers, &database).await?;
    if UserFormat::from_string(&target_user).is_some() {
        return Err(FydiaResponse::new_error_custom_status(
            "Soon may be",
            StatusCode::NOT_IMPLEMENTED,
        ));
    }

    let id = target_user
        .parse::<i32>()
        .map_err(|_| FydiaResponse::new_error("Bad user id"))?;

    let target = UserId::new(id)
        .get_user(&database)
        .await
        .ok_or_else(|| FydiaResponse::new_error("Bad user id"))?;

    let dm = DirectMessage::new(vec![user.id, target.id]);
    dm.insert(&database).await.map_err(|_| {
        FydiaResponse::new_error_custom_status(
            "Cannot insert in database",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(FydiaResponse::new_ok(""))
}
