use crate::sqlpool::parse_array;
use crate::sqlpool::FydiaPool;
use crate::sqlpool::ToAnyRows;
use fydia_struct::permission::Permission;
use fydia_struct::{roles::Role, server::ServerId};
use sqlx::Row;

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn get_roles_by_server_id(
        shortid: String,
        executor: &FydiaPool,
    ) -> Result<Vec<Role>, String>;
    async fn get_role_by_id(role_id: i32, executor: &FydiaPool) -> Result<(), String>;
    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String>;
    async fn update_color(&mut self, color: String, executor: &FydiaPool) -> Result<(), String>;
    async fn delete_role(&self, executor: &FydiaPool) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn get_roles_by_server_id(
        shortid: String,
        executor: &FydiaPool,
    ) -> Result<Vec<Self>, String> {
        let rawquery = "SELECT id, serverid, name, color, channel_access, permission FROM Roles WHERE serverid = ?;";
        let sqlresult = match executor {
            FydiaPool::Mysql(e) => match sqlx::query(rawquery).bind(shortid).fetch_all(e).await {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },

            FydiaPool::PgSql(e) => match sqlx::query(rawquery).bind(shortid).fetch_all(e).await {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
            FydiaPool::Sqlite(e) => match sqlx::query(rawquery).bind(shortid).fetch_all(e).await {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
        };
        let mut result = Vec::new();
        for i in sqlresult {
            result.push(Self {
                id: i.get("id"),
                server_id: ServerId::new(i.get("serverid")),
                name: i.get("name"),
                color: i.get("color"),
                channel_access: parse_array(i.get("channel_access")),
                permission: Permission::from_string(i.get("permission")),
            });
        }

        Ok(result)
    }

    async fn get_role_by_id(role_id: i32, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "SELECT * FROM Roles WHERE id = ?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery).bind(role_id).execute(mysql).await {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery).bind(role_id).execute(pgsql).await {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery).bind(role_id).execute(sqlite).await {
                    return Err(e.to_string());
                };
            }
        }

        Ok(())
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "UPDATE Roles SET name=? WHERE id=?";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        }

        self.name = name;

        Ok(())
    }

    async fn update_color(&mut self, color: String, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "UPDATE Roles SET color=? WHERE id=?";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&color)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&color)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&color)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        }

        self.color = color;

        Ok(())
    }

    async fn delete_role(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "DELETE FROM Roles WHERE id=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(mysql).await {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(pgsql).await {
                    return Err(e.to_string());
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(sqlite).await {
                    return Err(e.to_string());
                };
            }
        }

        Ok(())
    }
}
