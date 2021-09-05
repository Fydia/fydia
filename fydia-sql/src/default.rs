use sqlx::{query, Error};

use crate::sqlpool::FydiaPool;

pub async fn init(executor: &FydiaPool) -> Result<(), Error> {
    let mysql_query = include_str!("default/mysql.sql");
    let postgresql_query = include_str!("default/pgsql.sql");
    let sqlite_query = include_str!("default/sqlite.sql");
    match executor {
        FydiaPool::Mysql(mysql) => {
            for mysql_query in mysql_query.trim().split(';').collect::<Vec<&str>>() {
                if !mysql_query.is_empty() {
                    if let Err(e) = query(mysql_query).execute(mysql).await {
                        return Err(e);
                    }
                }
            }
        }
        FydiaPool::PgSql(mysql) => {
            if let Err(e) = query(postgresql_query).execute(mysql).await {
                return Err(e);
            }
        }
        FydiaPool::Sqlite(sqlite) => {
            if let Err(e) = query(sqlite_query).execute(sqlite).await {
                return Err(e);
            }
        }
    }

    Ok(())
}
