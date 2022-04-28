use axum::extract::Extension;
use fydia_sql::impls::channel::SqlDirectMessages;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ParentId;
use fydia_struct::response::FydiaResult;
use fydia_struct::{channel::DirectMessage, response::FydiaResponse};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Get all dm of an user
///
/// # Errors
/// This function will return an error if the token isn't valid
pub async fn get_direct_messages<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let user = BasicValues::get_user(&headers, &database).await?;
    let mut channels = DirectMessage::get_by_userid(&database, user.id)
        .await
        .map_err(|e| {
            error!("{e}");
            FydiaResponse::TextError("Error")
        })?;

    for i in channels.iter_mut() {
        if let ParentId::DirectMessage(direct_message) = &mut i.parent_id {
            if let Err(e) = direct_message.userid_to_user(&database).await {
                error!("{e}");
            };
        }
    }

    Ok(FydiaResponse::from_serialize(channels))
}
