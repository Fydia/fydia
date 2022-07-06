use migration::sea_orm::{DbConn, DbErr};
use std::sync::Arc;
/// Create default tables in database
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
pub async fn create_tables(db: &Arc<DbConn>) -> Result<(), DbErr> {
    migration::run_migrations(db).await
}
