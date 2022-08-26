use axum::body::Bytes;
use axum::extract::Extension;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResult, IntoFydia, MapError};
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
    let value = get_json_value_from_body(&body)?;
    let name = get_json("name", &value)?;

    let mut server = Server::new(name, user.id.clone()).error_to_fydiaresponse()?;

    server
        .insert(&database)
        .await
        .map(|_| server.id.id.into_ok())
}
