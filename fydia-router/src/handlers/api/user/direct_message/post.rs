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

/// Create a new direct message
///
/// # Errors
/// This function will return an error if body isn't valid or if the target isn't exist
pub async fn create_direct_message<'a>(
    headers: HeaderMap,
    Path(target_user): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let user = BasicValues::get_user(&headers, &database).await?;
    if UserFormat::from_string(&target_user).is_some() {
        return Err(FydiaResponse::TextErrorWithStatusCode(
            StatusCode::NOT_IMPLEMENTED,
            "Soon may be",
        ));
    }

    let id = target_user.parse::<i32>().map_err(|error| {
        error!("{error}");
        FydiaResponse::TextError("Bad user id")
    })?;

    let target = UserId::new(id)
        .get_user(&database)
        .await
        .ok_or(FydiaResponse::TextError("Bad user id"))?;

    let dm = DirectMessage::new(vec![user.id, target.id]);
    dm.insert(&database).await.map_err(|error| {
        error!("{error}");
        FydiaResponse::TextErrorWithStatusCode(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot insert in database",
        )
    })?;

    Ok(FydiaResponse::Text(""))
}
