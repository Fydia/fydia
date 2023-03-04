use fydia_sql::impls::server::SqlServer;

use crate::handlers::basic::{Database, UserFromToken};
use crate::handlers::{get_json, get_json_value_from_body};
use fydia_struct::response::{FydiaResult, IntoFydia, MapError};
use fydia_struct::server::Server;

/// Create a new server
///
/// # Errors
/// Return an error if body isn't valid or if database is unreachable
pub async fn create_server(
    UserFromToken(user): UserFromToken,
    Database(database): Database,
    body: String,
) -> FydiaResult {
    let value = get_json_value_from_body(&body)?;
    let name = get_json("name", &value)?;

    let mut server = Server::new(name, user.id.clone()).error_to_fydiaresponse()?;

    server
        .insert(&database)
        .await
        .map(|_| server.id.id.into_ok())
}
