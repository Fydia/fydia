use axum::body::Bytes;
use axum::extract::Extension;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResponse, FydiaResult};
use fydia_struct::server::Server;

use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};

/// Create a new server
///
/// # Errors
/// Return an error if body isn't valid or if database is unreachable
pub async fn create_server<'a>(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult<'a> {
    let user = BasicValues::get_user(&headers, &database).await?;
    let value = get_json_value_from_body(&body).map_err(FydiaResponse::StringError)?;
    let name = get_json("name", &value)?;

    let mut server = Server::new(name, user.id.clone()).map_err(FydiaResponse::StringError)?;

    server
        .insert_server(&database)
        .await
        .map(|_| FydiaResponse::String(server.id.id))
        .map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Cannot create the server")
        })
}
