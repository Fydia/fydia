use fydia_sql::impls::server::SqlServer;

use fydia_struct::response::{FydiaResult, IntoFydia};

use crate::handlers::basic::{Database, ServerFromId, UserFromToken};

/// Join a server
///
/// # Errors
/// Return an error if serverid doesn't exist
pub async fn join(
    UserFromToken(mut user): UserFromToken,
    ServerFromId(mut server): ServerFromId,
    Database(database): Database,
) -> FydiaResult {
    if user.servers.is_join(&server.id) {
        return Err("Already join".into_error());
    }

    server
        .join(&mut user, &database)
        .await
        .map(|_| "Server joined".into_ok())
}
