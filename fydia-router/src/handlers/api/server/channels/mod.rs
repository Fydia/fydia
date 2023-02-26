pub mod create;
pub mod delete;
pub mod messages;
pub mod permission;
pub mod typing;
pub mod update;
pub mod vocal;

use axum::extract::{Extension, Path};
use fydia_sql::{impls::channel::SqlChannel, sqlpool::DbConnection};
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::response::{FydiaResponse, FydiaResult};

/// Return requested channel
///
/// # Errors
/// Return an error if channelid isn't valid
pub async fn info_channel(
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    Channel::by_id(&ChannelId::new(channelid), &database)
        .await
        .map(FydiaResponse::from_serialize)
}
