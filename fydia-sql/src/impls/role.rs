use crate::sqlpool::parse_array;
use crate::sqlpool::FydiaPool;
use crate::sqlpool::ToAnyRows;
use fydia_struct::permission::Permission;
use fydia_struct::{roles::Role, server::ServerId};
use sqlx::Row;

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn get_roles_by_server_id(shortid: String, executor: &FydiaPool) -> Vec<Role>;
    async fn get_role_by_id(role_id: i32, executor: &FydiaPool);
    async fn update_name(&mut self, name: String, executor: &FydiaPool);
    async fn update_color(&mut self, color: String, executor: &FydiaPool);
    async fn delete_role(&self, executor: &FydiaPool);
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn get_roles_by_server_id(shortid: String, executor: &FydiaPool) -> Vec<Self> {
        let rawquery = "SELECT id, serverid, name, color, channel_access, permission FROM Roles WHERE serverid = ?;";
        let sqlresult = match executor {
            FydiaPool::Mysql(e) => sqlx::query(rawquery)
                .bind(shortid)
                .fetch_all(e)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::PgSql(e) => sqlx::query(rawquery)
                .bind(shortid)
                .fetch_all(e)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::Sqlite(e) => sqlx::query(rawquery)
                .bind(shortid)
                .fetch_all(e)
                .await
                .unwrap()
                .to_anyrows(),
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

        result
    }

    async fn get_role_by_id(role_id: i32, executor: &FydiaPool) {
        let rawquery = "SELECT * FROM Roles WHERE id = ?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(role_id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(role_id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(role_id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Roles SET name=? WHERE id=?";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }

        self.name = name;
    }

    async fn update_color(&mut self, color: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Roles SET color=? WHERE id=?";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&color)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&color)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&color)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }

        self.color = color;
    }

    async fn delete_role(&self, executor: &FydiaPool) {
        let rawquery = "DELETE FROM Roles WHERE id=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }
    }
}
