use super::message::SqlMessage;
use crate::sqlpool::FydiaPool;
use crate::sqlpool::{ToAnyRow, ToAnyRows};
use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType},
    messages::Message,
    server::{Channels, ServerId},
};
use logger::error;
use sqlx::Row;

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(id: ChannelId, executor: &FydiaPool) -> Option<Channel>;
    async fn get_channels_by_server_id(server_id: String, executor: &FydiaPool) -> Channels;
    async fn update_name(&mut self, name: String, executor: &FydiaPool);
    async fn update_description(&mut self, description: String, executor: &FydiaPool);
    async fn delete_channel(&self, executor: &FydiaPool);
    async fn get_messages(&self, executor: &FydiaPool) -> Vec<Message>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn get_channel_by_id(id: ChannelId, executor: &FydiaPool) -> Option<Channel> {
        let rawquery = "SELECT id, serverid, name, description, `type` FROM Channels WHERE id=?";
        let row = match executor {
            FydiaPool::Mysql(mysql) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(mysql).await {
                    Ok(r) => r.to_anyrow(),
                    Err(e) => {
                        error!(e.as_database_error().unwrap().to_string());
                        return None;
                    }
                }
            }
            FydiaPool::PgSql(pgsql) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(pgsql).await {
                    Ok(r) => r.to_anyrow(),
                    Err(e) => {
                        error!(e.as_database_error().unwrap().to_string());
                        return None;
                    }
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                match sqlx::query(rawquery).bind(&id.id).fetch_one(sqlite).await {
                    Ok(r) => r.to_anyrow(),
                    Err(e) => {
                        error!(e.as_database_error().unwrap().to_string());
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

    async fn get_channels_by_server_id(server_id: String, executor: &FydiaPool) -> Channels {
        let mut server_id = server_id;
        if server_id.len() > 10 {
            server_id = server_id.split_at(10).0.to_string();
        }

        let mut channels: Vec<Channel> = Vec::new();
        let rawquery = "SELECT * FROM Channels WHERE serverid = ?";
        let rows = match executor {
            FydiaPool::Mysql(mysql) => sqlx::query(rawquery)
                .bind(server_id)
                .fetch_all(mysql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::PgSql(pgsql) => sqlx::query(rawquery)
                .bind(server_id)
                .fetch_all(pgsql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::Sqlite(sqlite) => sqlx::query(rawquery)
                .bind(server_id)
                .fetch_all(sqlite)
                .await
                .unwrap()
                .to_anyrows(),
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

        Channels(channels)
    }

    async fn update_name(&mut self, name: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Channels SET name=? WHERE id=?;";

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

    async fn update_description(&mut self, description: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Channels SET description=? WHERE id=?;";

        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&description)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }

        self.description = description;
    }

    async fn delete_channel(&self, executor: &FydiaPool) {
        let rawquery = "DELETE FROM Channels WHERE id=?;";

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

    async fn get_messages(&self, executor: &FydiaPool) -> Vec<Message> {
        Message::get_messages_by_channel(self.id.clone(), executor)
            .await
            .unwrap()
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
