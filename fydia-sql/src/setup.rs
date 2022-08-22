use migration::sea_orm::{DbConn, DbErr};
use sea_orm::{ConnectionTrait, Statement};
use std::sync::Arc;

const RAW_SQL: &[&str] = &["ALTER TABLE Fydia.permission_roles ADD CONSTRAINT permission_roles_UN UNIQUE KEY (channel,value,`role`);",
                           "ALTER TABLE Fydia.permission_users ADD CONSTRAINT permission_users_UN UNIQUE KEY (`user`,channel,value);", 
                           "ALTER TABLE Fydia.roles_assignation ADD CONSTRAINT roles_assignation_UN UNIQUE KEY (role_id,user_id,server_id);"];

/// Create default tables in database
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
pub async fn create_tables(db: &Arc<DbConn>) -> Result<(), DbErr> {
    migration::run_migrations(db).await?;

    for i in RAW_SQL {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            i.to_string(),
        ))
        .await
        .map(|_| ())?;
    }

    Ok(())
}
