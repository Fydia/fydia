use axum::{
    body::Body,
    extract::{BodyStream, Extension, Path},
    response::IntoResponse,
};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::messages::Message;

use http::Request;

use crate::new_response;

pub async fn get_message_dm(
    request: Request<Body>,
    _body: BodyStream,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let res = new_response();
    let _headers = request.headers();

    println!(
        "{:?}",
        Message::get_messages_by_channel(dm_id.clone(), &database).await
    );

    res
}
