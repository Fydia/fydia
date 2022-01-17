use axum::{
    body::Body,
    extract::{Extension, Path},
    response::IntoResponse,
};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::{channel::ChannelId, messages::Message};

use http::Request;

use crate::new_response;

pub async fn get_message_dm(
    request: Request<Body>,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let res = new_response();
    let _headers = request.headers();

    println!(
        "{:?}",
        Message::get_messages_by_channel(ChannelId::new(dm_id.clone()), &database).await
    );

    res
}
