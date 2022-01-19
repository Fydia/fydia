pub mod create;
pub mod delete;
pub mod typing;
pub mod update;
pub mod vocal;

use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::{impls::channel::SqlChannel, sqlpool::DbConnection};

use fydia_struct::{
    channel::{Channel, ChannelId},
    response::FydiaResponse,
};

pub async fn info_channel(
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Some(channel) =
        Channel::get_channel_by_id(ChannelId::new(channelid.clone()), &database).await
    {
        return FydiaResponse::new_ok_json(&channel);
    }

    FydiaResponse::new_error("Error")
}
