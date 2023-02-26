use axum::extract::State;
use fydia_sql::impls::server::SqlServer;

use fydia_struct::response::{FydiaResult, IntoFydia, MapError};
use fydia_struct::server::Server;

use fydia_utils::http::HeaderMap;

use crate::handlers::basic::BasicValues;
use crate::handlers::{get_json, get_json_value_from_body};
use crate::ServerState;

/// Create a new server
///
/// # Errors
/// Return an error if body isn't valid or if database is unreachable
pub async fn create_server(
    headers: HeaderMap,
    State(state): State<ServerState>,
    body: String,
) -> FydiaResult {
    let user = BasicValues::get_user(&headers, &state.database).await?;
    let value = get_json_value_from_body(&body)?;
    let name = get_json("name", &value)?;

    let mut server = Server::new(name, user.id.clone()).error_to_fydiaresponse()?;

    server
        .insert(&state.database)
        .await
        .map(|_| server.id.id.into_ok())
}
