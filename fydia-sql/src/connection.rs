#![allow(clippy::expect_used)]

use std::process::exit;

use fydia_config::{DatabaseConfig, DatabaseType};
use sea_orm::{Database, DatabaseConnection};

pub async fn get_connection(configdatabase: &DatabaseConfig) -> DatabaseConnection {
    if configdatabase.database_type == DatabaseType::Sqlite
        && std::fs::File::open(format!("./{}", configdatabase.ip)).is_err()
    {
        std::fs::File::create(
            configdatabase
                .format_url()
                .strip_prefix("sqlite://")
                .map(|v| v.to_string())
                .unwrap_or_else(|| format!("{}.db", configdatabase.ip)),
        )
        .expect("Error");
    }
    match Database::connect(configdatabase.format_url().as_str()).await {
        Ok(e) => e,
        Err(e) => {
            error!(format!("{} => {}", configdatabase.format_url().as_str(), e));
            exit(0);
        }
    }
}
