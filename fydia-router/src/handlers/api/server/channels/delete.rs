use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannel;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::{Channel, ChannelId};
use fydia_struct::response::FydiaResponse;

use crate::new_response;

pub async fn delete_channel(
    Path((_, channelid)): Path<(String, String)>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();

    if let Some(channel) =
        Channel::get_channel_by_id(ChannelId::new(channelid.clone()), &database).await
    {
        if let Err(error) = channel.delete_channel(&database).await {
            error!(error);
            FydiaResponse::new_error(error).update_response(&mut res);
        };
    };

    res
}
