use super::message::SqlMessage;
use crate::sqlpool::FydiaPool;
use crate::sqlpool::{ToAnyRow, ToAnyRows};
use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType},
    messages::Message,
    server::{Channels, ServerId},
};
use sqlx::Row;

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(id: ChannelId, executor: &FydiaPool) -> Option<Channel>;
    async fn get_channels_by_server_id(
        server_id: String,
        executor: &FydiaPool,
    ) -> Result<Channels, String>;
    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String>;
    async fn update_description(
        &mut self,
        description: String,
        executor: &FydiaPool,
    ) -> Result<(), String>;
    async fn delete_channel(&self, executor: &FydiaPool) -> Result<(), String>;
    async fn get_messages(&self, executor: &FydiaPool) -> Result<Vec<Message>, String>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn get_channel_by_id(id: ChannelId, executor: &FydiaPool) -> Option<Channel> {
        let rawquery = "SELECT id, serverid, name, description, `type` FROM Channels WHERE id=?";
        let row = match executor {
            FydiaPool::Mysql(mysql) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(mysql).await {
                    Ok(r) => r.to_anyrow(),
                    Err(_) => {
                        return None;
                    }
                }
            }
            FydiaPool::PgSql(pgsql) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(pgsql).await {
                    Ok(r) => r.to_anyrow(),
                    Err(_) => {
                        return None;
                    }
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(sqlite).await {
                    Ok(r) => r.to_anyrow(),
                    Err(_) => {
                        return None;
                    }
                }
            }
        };

        return Some(Self {
            id: id.id,
            server_id: ServerId::new(row.get("serverid")),
            name: row.get("name"),
            description: row.get("description"),
            channel_type: ChannelType::from_string(row.get::<String, &str>("type")),
        });
    }

    async fn get_channels_by_server_id(
        server_id: String,
        executor: &FydiaPool,
    ) -> Result<Channels, String> {
        let mut server_id = server_id;
        if server_id.len() > 10 {
            server_id = server_id.split_at(10).0.to_string();
        }

        let mut channels: Vec<Channel> = Vec::new();
        let rawquery = "SELECT * FROM Channels WHERE serverid = ?";
        let rows = match executor {
            FydiaPool::Mysql(mysql) => {
                match sqlx::query(rawquery).bind(server_id).fetch_all(mysql).await {
                    Ok(e) => e.to_anyrows(),
                    Err(e) => return Err(e.to_string()),
                }
            }
            FydiaPool::PgSql(pgsql) => {
                match sqlx::query(rawquery).bind(server_id).fetch_all(pgsql).await {
                    Ok(e) => e.to_anyrows(),
                    Err(e) => return Err(e.to_string()),
                }
            }
            FydiaPool::Sqlite(sqlite) => match sqlx::query(rawquery)
                .bind(server_id)
                .fetch_all(sqlite)
                .await
            {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
        };

        for i in rows {
            channels.push(Self {
                id: i.get("id"),
                server_id: ServerId::new(i.get("serverid")),
                name: i.get("name"),
                description: i.get("description"),
                channel_type: ChannelType::from_string(i.get("type")),
            })
        }

        Ok(Channels(channels))
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "UPDATE Channels SET name=? WHERE id=?;";

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

    async fn update_description(
        &mut self,
        description: String,
        executor: &FydiaPool,
    ) -> Result<(), String> {
        let rawquery = "UPDATE Channels SET description=? WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        }

        self.description = description;

        Ok(())
    }

    async fn delete_channel(&self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "DELETE FROM Channels WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(mysql).await {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(pgsql).await {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery).bind(&self.id).execute(sqlite).await {
                    return Err(e.to_string());
                };
            }
        }

        Ok(())
    }

    async fn get_messages(&self, executor: &FydiaPool) -> Result<Vec<Message>, String> {
        Message::get_messages_by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn get_channel(&self, executor: &FydiaPool) -> Option<Channel>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn get_channel(&self, executor: &FydiaPool) -> Option<Channel> {
        Channel::get_channel_by_id(self.clone(), executor).await
    }
}
