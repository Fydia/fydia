use sea_orm::{DbConn, DbErr};

/// Create default tables in database
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
pub async fn create_tables(db: &DbConn) -> Result<(), DbErr> {
    migration::run_migrations(db).await
}
