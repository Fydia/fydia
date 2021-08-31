use fydia_struct::{
    channel::Channel,
    emoji::Emoji,
    instance::Instance,
    roles::Role,
    server::{Members, Server, ServerId, Servers},
    user::User,
};
use sqlx::Row;

use crate::sqlpool::{FydiaPool, ToAnyRow, ToAnyRows};

use super::{channel::SqlChannel, emoji::SqlEmoji, role::SqlRoles, user::SqlUser};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn get_user(&self, executor: &FydiaPool) -> Vec<User>;
    async fn get_server_by_id(id: ServerId, executor: &FydiaPool) -> Option<Server>;
    async fn insert_server(&self, executor: &FydiaPool);
    async fn delete_server(&self, executor: &FydiaPool);
    async fn update_name(&mut self, name: String, executor: &FydiaPool);
    async fn join(&mut self, mut user: User, executor: &FydiaPool);
    async fn insert_channel(&mut self, channel: Channel, executor: &FydiaPool);
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn get_user(&self, executor: &FydiaPool) -> Vec<User> {
        let mut result = Vec::new();

        let rawquery = format!(
            "SELECT * FROM User WHERE User.server LIKE '%{}%'",
            &self.shortid.clone()
        );
        let i = match &executor {
            FydiaPool::Mysql(mysqlpool) => sqlx::query(rawquery.as_str())
                .fetch_all(mysqlpool)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::PgSql(pgsqlpool) => sqlx::query(rawquery.as_str())
                .fetch_all(pgsqlpool)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::Sqlite(sqlitepool) => sqlx::query(rawquery.as_str())
                .fetch_all(sqlitepool)
                .await
                .unwrap()
                .to_anyrows(),
        };

        for user in i {
            result.push(User {
                id: user.get("id"),
                name: user.get("name"),
                instance: Instance::new(
                    fydia_struct::instance::Protocol::HTTP,
                    String::from("localhost"),
                    0,
                ),
                token: user.get("token"),
                email: user.get("email"),
                password: user.get("password"),
                description: user.get("description"),
                server: Servers(
                    serde_json::from_str(user.get::<String, &str>("server").as_str()).unwrap(),
                ),
            })
        }

        result
    }

    async fn get_server_by_id(id: ServerId, executor: &FydiaPool) -> Option<Server> {
        let rawquery = "SELECT * FROM Server WHERE shortid=? LIMIT 1;";
        let i = match &executor {
            FydiaPool::Mysql(mysqlpool) => sqlx::query(rawquery)
                .bind(id.short_id)
                .fetch_one(mysqlpool)
                .await
                .unwrap()
                .to_anyrow(),
            FydiaPool::PgSql(pgsqlpool) => sqlx::query(rawquery)
                .bind(id.short_id)
                .fetch_one(pgsqlpool)
                .await
                .unwrap()
                .to_anyrow(),
            FydiaPool::Sqlite(sqlitepool) => sqlx::query(rawquery)
                .bind(id.short_id)
                .fetch_one(sqlitepool)
                .await
                .unwrap()
                .to_anyrow(),
        };

        Some(Self {
            id: i.get("id"),
            shortid: i.get("shortid"),
            name: i.get("name"),
            owner: i.get("owner"),
            icon: i.get("icon"),
            members: serde_json::from_str::<Members>(i.get::<String, &str>("members").as_str())
                .unwrap(),
            roles: Role::get_roles_by_server_id(i.get("shortid"), executor).await,
            emoji: Emoji::get_emoji_by_server_id(i.get("shortid"), executor).await,
            channel: Channel::get_channels_by_server_id(i.get("shortid"), executor).await,
        })
    }
    async fn insert_server(&self, executor: &FydiaPool) {
        let rawquery = "INSERT INTO Server
                (id, shortid, name, owner, icon, members)
                VALUES(?, ?, ?, ?, ?, ?)";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(serde_json::to_string(&self.members).unwrap())
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(serde_json::to_string(&self.members).unwrap())
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(serde_json::to_string(&self.members).unwrap())
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        };
    }

    async fn delete_server(&self, executor: &FydiaPool) {
        let rawquery = "DELETE FROM Server WHERE shortid=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(sqlite)
                    .await
                    .expect("Error on delete server");
            }
        }
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Server SET name=? WHERE shortid=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
                    .execute(sqlite)
                    .await
                    .expect("Error on delete server");
            }
        }

        self.name = name;
    }

    async fn join(&mut self, mut user: User, executor: &FydiaPool) {
        let mut vecuser = self.get_user(executor).await;
        vecuser.push(user.clone());

        let value = Members::new_with(vecuser.len() as i32, vecuser);

        let rawquery = "UPDATE Server SET members=? WHERE shortid=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(serde_json::to_string(&value).unwrap())
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(serde_json::to_string(&value).unwrap())
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(serde_json::to_string(&value).unwrap())
                    .bind(&self.shortid)
                    .execute(sqlite)
                    .await
                    .expect("Error on delete server");
            }
        }
        user.insert_server(ServerId::new(self.id.clone()), executor)
            .await
            .expect("Error");
        self.members = value;
    }

    async fn insert_channel(&mut self, channel: Channel, executor: &FydiaPool) {
        let rawquery =
            "INSERT INTO Channels (id, serverid, name, description, `type`) VALUES(?, ?, ?, ?, ?);";
        let to_push = channel.clone();
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(mysql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(pgsql)
                    .await
                    .expect("Error on delete server");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(sqlite)
                    .await
                    .expect("Error on delete server");
            }
        }

        self.channel.0.push(to_push.clone());
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get_server(&self, executor: &FydiaPool) -> Option<Server>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get_server(&self, executor: &FydiaPool) -> Option<Server> {
        Server::get_server_by_id(ServerId::new(self.id.clone()), executor).await
    }
}
