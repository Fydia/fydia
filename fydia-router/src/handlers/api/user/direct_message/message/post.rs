use axum::{
    body::Body,
    extract::{Extension, Path},
    http::Request,
    response::IntoResponse,
};
use fydia_sql::{impls::message::SqlMessage, sqlpool::DbConnection};
use fydia_struct::messages::Message;

use crate::new_response;

pub async fn post_message_dm(
    response: Request<Body>,
    Path(dm_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let res = new_response();
    let _headers = response.headers();

    println!(
        "{:?}",
        Message::get_messages_by_channel(dm_id.clone(), &database).await
    );
    res
}
