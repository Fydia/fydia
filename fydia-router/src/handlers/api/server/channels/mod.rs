pub mod create;
pub mod delete;
pub mod messages;
pub mod typing;
pub mod update;
pub mod vocal;

use axum::extract::{Extension, Path};
use fydia_sql::{impls::channel::SqlChannel, sqlpool::DbConnection};

use fydia_struct::response::FydiaResult;
use fydia_struct::{
    channel::{Channel, ChannelId},
    response::FydiaResponse,
};

pub async fn info_channel(
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> FydiaResult {
    let channel = Channel::get_channel_by_id(&ChannelId::new(channelid), &database)
        .await
        .map_err(FydiaResponse::new_error)?;

    Ok(FydiaResponse::new_ok_json(&channel))
}
