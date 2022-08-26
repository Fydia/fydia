use axum::extract::{Extension, Path};
use fydia_sql::impls::direct_message::{DirectMessageMembers, SqlDirectMessage};
use fydia_sql::impls::user::UserFrom;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::directmessage::DirectMessage;
use fydia_struct::response::{FydiaResult, IntoFydia, MapError};
use fydia_struct::utils::Id;
use fydia_struct::{format::UserFormat, user::UserId};
use fydia_utils::http::HeaderMap;

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
        return Err("Soon may be".into_not_implemented_error());
    }

    let id = target_user.parse::<u32>().error_to_fydiaresponse()?;

    let target = UserId::new(id).to_user(&database).await?;

    let mut dm = DirectMessage::new(Id::Unset, "New DM channel".to_string(), "".to_string());
    dm.insert(&database).await?;

    dm.add(&user.id, &database).await?;

    dm.add(&target.id, &database).await?;

    Ok("".into_error())
}
