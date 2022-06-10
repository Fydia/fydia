#![allow(rust_2018_idioms)]

pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbConn;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}

/// Run migrations
///
/// # Errors
/// Return an error if :
/// * Database is unavailable
/// * Migration is wrong
pub async fn run_migrations(db: &DbConn) -> Result<(), DbErr> {
    Migrator::up(db, None).await
}
