use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn join(
    headers: HeaderMap,
    Path(server_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let (mut user, mut server) =
        match BasicValues::get_user_and_server(&headers, server_id, &database).await {
            Ok(v) => v,
            Err(error) => return FydiaResponse::new_error(error),
        };

    return if user.servers.is_join(&server.id) {
        FydiaResponse::new_error("Already join")
    } else if let Err(error) = server.join(&mut user, &database).await {
        error!(error);
        FydiaResponse::new_error("Cannot join")
    } else {
        FydiaResponse::new_ok("Server joined")
    };
}
