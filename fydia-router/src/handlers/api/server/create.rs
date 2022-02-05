use axum::body::Bytes;
use axum::extract::Extension;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::server::Server;

use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;
use crate::handlers::get_json;

pub async fn create_server(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let user = BasicValues::get_user(&headers, &database).await?;

    let body =
        String::from_utf8(body.to_vec()).map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let value = serde_json::from_str::<Value>(body.as_str())
        .map_err(|_| FydiaResponse::new_error("Bad Body"))?;

    let name = get_json("name", &value)?;

    let mut server = Server::new(name, user.id.clone());

    server
        .insert_server(&database)
        .await
        .map(|_| FydiaResponse::new_ok(server.id.id))
        .map_err(|_| FydiaResponse::new_error("Cannot join the server"))
}
