use axum::extract::{Extension, Path};
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::{FydiaResult, IntoFydia};
use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;

/// Join a server
///
/// # Errors
/// Return an error if serverid doesn't exist
pub async fn join(
    headers: HeaderMap,
    Path(server_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> FydiaResult {
    let (mut user, mut server) =
        BasicValues::get_user_and_server(&headers, server_id, &database).await?;

    if user.servers.is_join(&server.id) {
        return Err("Already join".into_error());
    }

    server
        .join(&mut user, &database)
        .await
        .map(|_| "Server joined".into_ok())
}
