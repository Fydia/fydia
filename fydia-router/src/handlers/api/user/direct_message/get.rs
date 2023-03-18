use crate::handlers::basic::Database;
use crate::handlers::basic::UserFromToken;
use fydia_sql::impls::direct_message::DirectMessageMembers;
use fydia_struct::directmessage::DirectMessage;
use fydia_struct::response::FydiaResponse;
use fydia_struct::response::FydiaResult;

/// Get all dm of an user
///
/// # Errors
/// This function will return an error if the token isn't valid
pub async fn get_direct_messages(
    UserFromToken(user): UserFromToken,
    Database(database): Database,
) -> FydiaResult {
    let channels = DirectMessage::of_user(&user.id, &database).await?;
    FydiaResponse::from_serialize(channels).into()
}
