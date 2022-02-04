use sea_orm::{ConnectionTrait, DbConn, DbErr, Statement};

pub async fn create_tables(db: &DbConn) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    match builder {
        sea_orm::DatabaseBackend::MySql => {
            let queries = include_str!("migrations/mysql.sql").to_string().to_string();
            for mysql_query in queries.trim().split(';').collect::<Vec<&str>>() {
                if !mysql_query.is_empty() {
                    db.execute(Statement::from_string(
                        sea_orm::DatabaseBackend::MySql,
                        mysql_query.to_string(),
                    ))
                    .await?;
                }
            }
        }
        sea_orm::DatabaseBackend::Postgres => {
            todo!()
        }
        sea_orm::DatabaseBackend::Sqlite => {
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                include_str!("migrations/sqlite.sql")
                    .to_string()
                    .to_string(),
            ))
            .await?;
        }
    }

    Ok(())
}
