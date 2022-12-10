#![allow(clippy::expect_used)]

use fydia_config::{DatabaseConfig, DatabaseType};
use sea_orm::{Database, DatabaseConnection};
use shared::sea_orm;

pub async fn get_connection(configdatabase: &DatabaseConfig) -> DatabaseConnection {
    if configdatabase.database_type == DatabaseType::Sqlite
        && std::fs::File::open(format!("./{}", configdatabase.ip)).is_err()
    {
        std::fs::File::create(
            configdatabase
                .format_url()
                .strip_prefix("sqlite://")
                .map_or_else(|| format!("{}.db", configdatabase.ip), |v| v.to_string()),
        )
        .expect("Error");
    }
    match Database::connect(configdatabase.format_url().as_str()).await {
        Ok(e) => e,
        Err(e) => {
            error!("{} => {}", configdatabase.format_url().as_str(), e);
            panic!("Cannot get a connection with database");
        }
    }
}
