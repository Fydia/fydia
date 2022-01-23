use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub type DbConnection = Arc<DatabaseConnection>;