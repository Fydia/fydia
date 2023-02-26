use axum::extract::{Extension, Path, State};
use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResult, IntoFydia, MapError};

use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};
use crate::ServerState;

/// Change name of a channel
///
/// # Errors
/// Return an error if serverid or channelid or body isn't valid
pub async fn update_name(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
    body: String,
) -> FydiaResult {
    let (user, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
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

    let json = get_json_value_from_body(&body)?;

    let name = get_json("name", &json)?;

    channel
        .update_name(name, &database)
        .await
        .map(|_| "Channel name updated".into_ok())
}

/// Change description of a channel
///
/// # Errors
/// Return an error if channelid or serverid or body isn't valid
pub async fn update_description(
    State(state): State<ServerState>,
    Path((serverid, channelid)): Path<(String, String)>,
    headers: HeaderMap,
    body: String,
) -> FydiaResult {
    let (_, _, mut channel) = BasicValues::get_user_and_server_and_check_if_joined_and_channel(
        &headers,
        &serverid,
        &channelid,
        &state.database,
    )
    .await?;

    let json = get_json_value_from_body(&body)?;

    let description = get_json("description", &json)?;

    channel
        .update_description(description, &state.database)
        .await
        .map(|_| "Channel description updated".into_ok())
}
