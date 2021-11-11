pub mod create;
pub mod delete;
pub mod update;
pub mod vocal;

use axum::http::Request;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{Extension, Path},
};
use fydia_sql::{impls::channel::SqlChannel, sqlpool::DbConnection};

use fydia_struct::{
    channel::{Channel, ChannelId},
    response::FydiaResponse,
};

use crate::new_response;

pub async fn info_channel(
    _request: Request<Body>,
    Extension(database): Extension<DbConnection>,
    Path((_serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut res = new_response();
    if let Some(channel) =
        Channel::get_channel_by_id(ChannelId::new(channelid.clone()), &database).await
    {
        FydiaResponse::new_ok_json(&channel).update_response(&mut res);
    } else {
        FydiaResponse::new_error("Error").update_response(&mut res);
    }
    res
}
