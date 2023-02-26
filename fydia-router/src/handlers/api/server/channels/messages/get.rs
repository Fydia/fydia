use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::impls::{channel::SqlChannel, user::SqlUser};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult, IntoFydia, MapError};
use fydia_utils::http::HeaderMap;

/// Return all message of channel
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn get_messages(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    if !user
        .permission_of_channel(&channel.id, &database)
        .await?
        .calculate(Some(channel.id.clone()))
        .error_to_fydiaresponse()?
        .can_read()
    {
        return FydiaResult::Err("Unknow channel".into_error());
    }

    channel
        .messages(&database)
        .await
        .map(|value| FydiaResponse::from_serialize(value))
}
