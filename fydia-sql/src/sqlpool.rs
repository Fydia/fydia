use sea_orm::DatabaseConnection;
use shared::sea_orm;
use std::sync::Arc;

pub type DbConnection = Arc<DatabaseConnection>;
