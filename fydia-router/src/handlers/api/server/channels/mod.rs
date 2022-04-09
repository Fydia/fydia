pub mod create;
pub mod delete;
pub mod messages;
pub mod typing;
pub mod update;
pub mod vocal;

use axum::extract::{Extension, Path};
use fydia_sql::{impls::channel::SqlChannel, sqlpool::DbConnection};
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::response::{FydiaResponse, FydiaResult};

pub async fn info_channel<'a>(
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> FydiaResult<'a> {
    Channel::get_channel_by_id(&ChannelId::new(channelid), &database)
        .await
        .map(FydiaResponse::from_serialize)
        .map_err(FydiaResponse::StringError)
}
