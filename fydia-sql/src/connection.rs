#![allow(clippy::expect_used)]

use fydia_config::DatabaseConfig;
use sea_orm::{Database, DatabaseConnection};

pub async fn get_connection(configdatabase: &DatabaseConfig) -> DatabaseConnection {
    /*let a = match configdatabase.database_type {
        DatabaseType::Mysql =>
            MySqlPool::connect(configdatabase.format_url().as_str())
                .await
                .expect("Error"),
        DatabaseType::PgSql =>
            PgPool::connect(configdatabase.format_url().as_str())
                .await
                .expect("Error"),
        DatabaseType::Sqlite => {
            std::fs::File::create(&configdatabase.ip).expect("Error");
                SqlitePool::connect(configdatabase.ip.as_str())
                    .await
                    .expect("Error")
        }
    }*/
    Database::connect(configdatabase.format_url().as_str())
        .await
        .unwrap()
}
