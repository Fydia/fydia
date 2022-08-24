use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use fydia_sql::impls::{channel::SqlChannel, user::SqlUser};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_utils::http::HeaderMap;

/// Return all message of channel
///
/// # Errors
/// Return an error if:
/// * serverid, channelid, token isn't valid
/// * database is unreachable
pub async fn get_messages<'a>(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    let (user, _, channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers, &serverid, &channelid, &database,
    )
    .await?;

    if !user
        .permission_of_channel(&channel.id, &database)
        .await
        .map_err(|_err| FydiaResponse::TextError("Cannot get permission"))?
        .calculate(Some(channel.id.clone()))
        .map_err(FydiaResponse::StringError)?
        .can_read()
    {
        return FydiaResult::Err(FydiaResponse::TextError("Unknow channel"));
    }

    channel
        .messages(&database)
        .await
        .map(|value| FydiaResponse::from_serialize(&value))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot get message")
        })
}
