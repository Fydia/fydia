#![allow(clippy::expect_used)]

use std::process::exit;

use fydia_config::{DatabaseConfig, DatabaseType};
use sea_orm::{Database, DatabaseConnection};

pub async fn get_connection(configdatabase: &DatabaseConfig) -> DatabaseConnection {
    if configdatabase.database_type == DatabaseType::Sqlite {
        std::fs::File::create(&configdatabase.ip).expect("Error");
    }
    match Database::connect(configdatabase.format_url().as_str()).await {
        Ok(e) => e,
        Err(e) => {
            error!(e.to_string());
            exit(0);
        }
    }
}
