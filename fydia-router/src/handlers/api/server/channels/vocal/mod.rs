use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::Request;
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlChannelId;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::{channel::ChannelId, server::ServerId};

use crate::new_response;

pub async fn join_channel(
    request: Request<Body>,
    Extension(database): Extension<DbConnection>,
    Path((serverid, channelid)): Path<(String, String)>,
) -> impl IntoResponse {
    let _headers = request.headers();
    let _serverid = ServerId::new(serverid.clone());
    let channelid = ChannelId::new(channelid.clone());
    let mut res = new_response();

    if let Some(channel) = channelid.get_channel(&database).await {
        if channel.channel_type.is_voice() {
            FydiaResponse::new_ok("Vocal Channel").update_response(&mut res);
        } else {
            FydiaResponse::new_ok("Text Channel").update_response(&mut res);
        }
    };

    res
}
