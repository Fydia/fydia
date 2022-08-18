use axum::extract::Extension;
use fydia_sql::impls::direct_message::DirectMessageMembers;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::directmessage::DirectMessage;
use fydia_struct::response::FydiaResponse;
use fydia_struct::response::FydiaResult;
use fydia_utils::http::HeaderMap;

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
    let channels = DirectMessage::of_user(&user.id, &database)
        .await
        .map_err(|e| {
            error!("{e}");
            FydiaResponse::TextError("Error")
        })?;
    Ok(FydiaResponse::from_serialize(channels))
}
