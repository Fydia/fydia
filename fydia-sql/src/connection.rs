use crate::sqlpool::FydiaPool;
use fydia_config::{DatabaseConfig, DatabaseType};
use sqlx::{MySqlPool, PgPool, SqlitePool};

pub async fn get_connection(configdatabase: &DatabaseConfig) -> FydiaPool {
    match configdatabase.database_type {
        DatabaseType::Mysql => FydiaPool::Mysql(
            MySqlPool::connect(configdatabase.format_url().as_str())
                .await
                .expect("Error"),
        ),
        DatabaseType::PgSql => FydiaPool::PgSql(
            PgPool::connect(configdatabase.format_url().as_str())
                .await
                .expect("Error"),
        ),
        DatabaseType::Sqlite => FydiaPool::Sqlite(
            SqlitePool::connect(configdatabase.format_url().as_str())
                .await
                .expect("Error"),
        ),
    }
}
