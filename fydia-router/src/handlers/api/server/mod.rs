use crate::handlers::basic::BasicValues;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

pub mod channels;
pub mod create;
pub mod info;
pub mod join;
pub mod picture;
pub mod roles;

pub async fn get_server(
    headers: HeaderMap,
    Path(serverid): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    match BasicValues::get_user_and_server_and_check_if_joined(&headers, serverid, &database).await
    {
        Ok((_, server)) => FydiaResponse::new_ok_json(&server),
        Err(error) => error,
    }
}
