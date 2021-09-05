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
    async fn get_user(&self, executor: &FydiaPool) -> Result<Vec<User>, String>;
    async fn get_server_by_id(id: ServerId, executor: &FydiaPool) -> Result<Server, String>;
    async fn insert_server(&self, executor: &FydiaPool) -> Result<(), String>;
    async fn delete_server(&self, executor: &FydiaPool) -> Result<(), String>;
    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String>;
    async fn join(&mut self, mut user: User, executor: &FydiaPool) -> Result<(), String>;
    async fn insert_channel(
        &mut self,
        channel: Channel,
        executor: &FydiaPool,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn get_user(&self, executor: &FydiaPool) -> Result<Vec<User>, String> {
        let mut result = Vec::new();

        let rawquery = format!(
            "SELECT * FROM User WHERE User.server LIKE '%{}%'",
            &self.shortid.clone()
        );
        let i = match &executor {
            FydiaPool::Mysql(mysqlpool) => {
                match sqlx::query(rawquery.as_str()).fetch_all(mysqlpool).await {
                    Ok(e) => e.to_anyrows(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
            FydiaPool::PgSql(pgsqlpool) => {
                match sqlx::query(rawquery.as_str()).fetch_all(pgsqlpool).await {
                    Ok(e) => e.to_anyrows(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
            FydiaPool::Sqlite(sqlitepool) => {
                match sqlx::query(rawquery.as_str()).fetch_all(sqlitepool).await {
                    Ok(e) => e.to_anyrows(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
        };

        for user in i {
            let json = match serde_json::from_str(user.get::<String, &str>("server").as_str()) {
                Ok(json) => json,
                Err(e) => return Err(e.to_string()),
            };

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
                server: Servers(json),
            })
        }

        Ok(result)
    }

    async fn get_server_by_id(id: ServerId, executor: &FydiaPool) -> Result<Server, String> {
        let rawquery = "SELECT * FROM Server WHERE shortid=? LIMIT 1;";
        let i = match &executor {
            FydiaPool::Mysql(mysqlpool) => {
                match sqlx::query(rawquery)
                    .bind(id.short_id)
                    .fetch_one(mysqlpool)
                    .await
                {
                    Ok(e) => e.to_anyrow(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
            FydiaPool::PgSql(pgsqlpool) => {
                match sqlx::query(rawquery)
                    .bind(id.short_id)
                    .fetch_one(pgsqlpool)
                    .await
                {
                    Ok(e) => e.to_anyrow(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
            FydiaPool::Sqlite(sqlitepool) => {
                match sqlx::query(rawquery)
                    .bind(id.short_id)
                    .fetch_one(sqlitepool)
                    .await
                {
                    Ok(e) => e.to_anyrow(),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
        };

        let members =
            match serde_json::from_str::<Members>(i.get::<String, &str>("members").as_str()) {
                Ok(e) => e,
                Err(e) => return Err(e.to_string()),
            };
        let roles = match Role::get_roles_by_server_id(i.get("shortid"), executor).await {
            Ok(e) => e,
            Err(e) => return Err(e),
        };

        let channel = match Channel::get_channels_by_server_id(i.get("shortid"), executor).await {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        Ok(Self {
            id: i.get("id"),
            shortid: i.get("shortid"),
            name: i.get("name"),
            owner: i.get("owner"),
            icon: i.get("icon"),
            members,
            roles,
            emoji: Emoji::get_emoji_by_server_id(i.get("shortid"), executor).await,
            channel,
        })
    }
    async fn insert_server(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "INSERT INTO Server
                (id, shortid, name, owner, icon, members)
                VALUES(?, ?, ?, ?, ?, ?)";
        let json = match serde_json::to_string(&self.members) {
            Ok(json) => json,
            Err(error) => return Err(error.to_string()),
        };
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(json)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(json)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.shortid)
                    .bind(&self.name)
                    .bind(self.owner)
                    .bind(&self.icon)
                    .bind(json)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        };

        Ok(())
    }

    async fn delete_server(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "DELETE FROM Server WHERE shortid=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.shortid)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        }

        Ok(())
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "UPDATE Server SET name=? WHERE shortid=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&name)
                    .bind(&self.shortid)
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

    async fn join(&mut self, mut user: User, executor: &FydiaPool) -> Result<(), String> {
        let mut vecuser = match self.get_user(executor).await {
            Ok(vec_users) => vec_users,
            Err(e) => return Err(e),
        };
        vecuser.push(user.clone());

        let value = Members::new_with(vecuser.len() as i32, vecuser);
        let json = match serde_json::to_string(&value) {
            Ok(json) => json,
            Err(e) => return Err(e.to_string()),
        };
        let rawquery = "UPDATE Server SET members=? WHERE shortid=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.shortid)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.shortid)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(json)
                    .bind(&self.shortid)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        }
        if let Err(error) = user
            .insert_server(ServerId::new(self.id.clone()), executor)
            .await
        {
            return Err(error);
        }

        self.members = value;

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: Channel,
        executor: &FydiaPool,
    ) -> Result<(), String> {
        let rawquery =
            "INSERT INTO Channels (id, serverid, name, description, `type`) VALUES(?, ?, ?, ?, ?);";
        let to_push = channel.clone();
        match executor {
            FydiaPool::Mysql(mysql) => {
                if sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(mysql)
                    .await
                    .is_err()
                {
                    return Err("Cannot insert server".to_string());
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(pgsql)
                    .await
                    .is_err()
                {
                    return Err("Cannot insert server".to_string());
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if sqlx::query(rawquery)
                    .bind(channel.id)
                    .bind(&self.shortid)
                    .bind(channel.name)
                    .bind(channel.description)
                    .bind(channel.channel_type.to_string())
                    .execute(sqlite)
                    .await
                    .is_err()
                {
                    return Err("Cannot insert server".to_string());
                }
            }
        }

        self.channel.0.push(to_push.clone());

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get_server(&self, executor: &FydiaPool) -> Result<Server, String>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get_server(&self, executor: &FydiaPool) -> Result<Server, String> {
        Server::get_server_by_id(ServerId::new(self.id.clone()), executor).await
    }
}
