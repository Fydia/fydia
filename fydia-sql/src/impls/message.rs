use crate::sqlpool::{FydiaPool, ToAnyRows};
use fydia_struct::{
    channel::ChannelId,
    messages::{datetime_to_sqltime, Message, MessageType, SqlDate},
    user::User,
};
use sqlx::Row;

use super::user::SqlUser;

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn get_messages_by_user_id(id: i32, executor: &FydiaPool)
        -> Result<Vec<Message>, String>;
    async fn get_messages_by_channel(
        channel_id: String,
        executor: &FydiaPool,
    ) -> Result<Vec<Message>, String>;
    async fn insert_message(&mut self, executor: &FydiaPool) -> Result<(), String>;
    async fn update_message(&mut self, content: String, executor: &FydiaPool)
        -> Result<(), String>;
    async fn delete_message(&mut self, executor: &FydiaPool) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn get_messages_by_user_id(
        id: i32,
        executor: &FydiaPool,
    ) -> Result<Vec<Message>, String> {
        let mut messages: Vec<Message> = Vec::new();
        let rawquery = "SELECT * FROM Messages WHERE author_id=? ORDER BY `timestamp` LIMIT 50";
        let result = match executor {
            FydiaPool::Mysql(mysql) => {
                match sqlx::query(rawquery).bind(id).fetch_all(mysql).await {
                    Ok(rows) => rows.to_anyrows(),
                    Err(e) => return Err(e.to_string()),
                }
            }
            FydiaPool::PgSql(pgsql) => {
                match sqlx::query(rawquery).bind(id).fetch_all(pgsql).await {
                    Ok(rows) => rows.to_anyrows(),
                    Err(e) => return Err(e.to_string()),
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                match sqlx::query(rawquery).bind(id).fetch_all(sqlite).await {
                    Ok(rows) => rows.to_anyrows(),
                    Err(e) => return Err(e.to_string()),
                }
            }
        };

        for i in result {
            let author_id = match User::get_user_by_id(i.get("author_id"), executor).await {
                Some(author_id) => author_id,
                None => return Err("Error Author_id".to_string()),
            };

            let message_type = match MessageType::from_string(i.get("message_type")) {
                Some(e) => e,
                None => return Err("Error Message_type".to_string()),
            };

            messages.push(Message {
                id: i.get("id"),
                content: i.get("content"),
                message_type,
                edited: i.get::<bool, &str>("edited"),
                timestamp: SqlDate::parse_string(i.get("timestamp")).unwrap_or_else(SqlDate::null),
                channel_id: ChannelId::new(i.get("channel_id")),
                author_id,
            })
        }

        Ok(messages)
    }

    async fn get_messages_by_channel(
        channel_id: String,
        executor: &FydiaPool,
    ) -> Result<Vec<Message>, String> {
        let mut messages: Vec<Message> = Vec::new();
        let rawquery = "SELECT * FROM Messages WHERE channel_id=? ORDER BY `timestamp` LIMIT 50";
        let result = match executor {
            FydiaPool::Mysql(mysql) => match sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(mysql)
                .await
            {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
            FydiaPool::PgSql(pgsql) => match sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(pgsql)
                .await
            {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
            FydiaPool::Sqlite(sqlite) => match sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(sqlite)
                .await
            {
                Ok(e) => e.to_anyrows(),
                Err(e) => return Err(e.to_string()),
            },
        };

        for i in result {
            let author_id = match User::get_user_by_id(i.get("author_id"), executor).await {
                Some(e) => e,
                None => return Err(String::from("Author_id error")),
            };

            let message_type = match MessageType::from_string(i.get("message_type")) {
                Some(e) => e,
                None => return Err("Message_type error".to_string()),
            };

            messages.push(Message {
                id: i.get("id"),
                content: i.get("content"),
                message_type,
                edited: i.get::<bool, &str>("edited"),
                timestamp: SqlDate::parse_string(i.get("timestamp")).unwrap_or_else(SqlDate::null),
                channel_id: ChannelId::new(i.get("channel_id")),
                author_id,
            })
        }

        Ok(messages)
    }

    async fn insert_message(&mut self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery ="INSERT INTO Messages (id, content, message_type, edited, `timestamp`, channel_id, author_id) VALUES(?, ?, ?, ?, ?, ?, ?);";

        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                };
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                };
            }
        };

        Ok(())
    }

    async fn update_message(
        &mut self,
        content: String,
        executor: &FydiaPool,
    ) -> Result<(), String> {
        let rawquery = "UPDATE Messages SET content=?, edited=TRUE, WHERE id=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                {
                    return Err(e.to_string());
                }
            }
            FydiaPool::PgSql(pgsql) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                {
                    return Err(e.to_string());
                }
            }
            FydiaPool::Sqlite(sqlite) => {
                if let Err(e) = sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                {
                    return Err(e.to_string());
                }
            }
        }

        self.content = content;

        Ok(())
    }

    async fn delete_message(&mut self, executor: &FydiaPool) -> Result<(), String> {
        let rawquery = "DELETE FROM Messages WHERE id=?;";
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
}
