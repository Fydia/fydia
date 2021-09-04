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
    async fn insert_message(&mut self, executor: &FydiaPool);
    async fn update_message(&mut self, content: String, executor: &FydiaPool);
    async fn delete_message(&mut self, executor: &FydiaPool);
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
            FydiaPool::Mysql(mysql) => sqlx::query(rawquery)
                .bind(id)
                .fetch_all(mysql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::PgSql(pgsql) => sqlx::query(rawquery)
                .bind(id)
                .fetch_all(pgsql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::Sqlite(sqlite) => sqlx::query(rawquery)
                .bind(id)
                .fetch_all(sqlite)
                .await
                .unwrap()
                .to_anyrows(),
        };

        for i in result {
            messages.push(Message {
                id: i.get("id"),
                content: i.get("content"),
                message_type: MessageType::from_string(i.get("message_type")).unwrap(),
                edited: i.get::<bool, &str>("edited"),
                timestamp: SqlDate::parse_string(i.get("timestamp")).unwrap_or(SqlDate::null()),
                channel_id: ChannelId::new(i.get("channel_id")),
                author_id: User::get_user_by_id(i.get("author_id"), executor)
                    .await
                    .unwrap(),
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
            FydiaPool::Mysql(mysql) => sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(mysql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::PgSql(pgsql) => sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(pgsql)
                .await
                .unwrap()
                .to_anyrows(),
            FydiaPool::Sqlite(sqlite) => sqlx::query(rawquery)
                .bind(channel_id)
                .fetch_all(sqlite)
                .await
                .unwrap()
                .to_anyrows(),
        };

        for i in result {
            messages.push(Message {
                id: i.get("id"),
                content: i.get("content"),
                message_type: MessageType::from_string(i.get("message_type")).unwrap(),
                edited: i.get::<bool, &str>("edited"),
                timestamp: SqlDate::parse_string(i.get("timestamp")).unwrap_or(SqlDate::null()),
                channel_id: ChannelId::new(i.get("channel_id")),
                author_id: User::get_user_by_id(i.get("author_id"), executor)
                    .await
                    .unwrap(),
            })
        }

        Ok(messages)
    }

    async fn insert_message(&mut self, executor: &FydiaPool) {
        let rawquery ="INSERT INTO Messages (id, content, message_type, edited, `timestamp`, channel_id, author_id) VALUES(?, ?, ?, ?, ?, ?, ?);";

        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&self.id)
                    .bind(&self.content)
                    .bind(&self.message_type.to_string())
                    .bind(&self.edited)
                    .bind(datetime_to_sqltime(self.timestamp.0))
                    .bind(&self.channel_id.id)
                    .bind(&self.author_id.id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }
    }

    async fn update_message(&mut self, content: String, executor: &FydiaPool) {
        let rawquery = "UPDATE Messages SET content=?, edited=TRUE, WHERE id=?;";
        match executor {
            FydiaPool::Mysql(mysql) => {
                sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(mysql)
                    .await
                    .expect("Error");
            }
            FydiaPool::PgSql(pgsql) => {
                sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(pgsql)
                    .await
                    .expect("Error");
            }
            FydiaPool::Sqlite(sqlite) => {
                sqlx::query(rawquery)
                    .bind(&content)
                    .bind(&self.id)
                    .execute(sqlite)
                    .await
                    .expect("Error");
            }
        }

        self.content = content;
    }

    async fn delete_message(&mut self, executor: &FydiaPool) {
        let rawquery = "DELETE FROM Messages WHERE id=?;";
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
