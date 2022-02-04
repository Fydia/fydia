#![allow(irrefutable_let_patterns)]

use super::user::SqlUser;
use fydia_struct::{
    channel::ChannelId,
    messages::{Date, Message, MessageType},
    user::User,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn get_messages_by_user_id(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String>;
    async fn get_messages_by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String>;
    async fn insert_message(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_message<T: Into<String> + Send>(
        &mut self,
        content: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn delete_message(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn get_messages_by_user_id(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let mut query = crate::entity::messages::Entity::find()
            .filter(crate::entity::messages::Column::AuthorId.eq(id))
            .order_by(
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            )
            .paginate(executor, 50);
        while let Ok(Some(e)) = query.fetch_and_next().await {
            for i in e {
                let author_id = User::get_user_by_id(i.author_id, executor)
                    .await
                    .ok_or_else(|| "Error Author_id".to_string())?;

                let message_type = MessageType::from_string(i.message_type)
                    .ok_or_else(|| "Error Message_type".to_string())?;

                messages.push(Message {
                    id: i.id,
                    content: i.content.unwrap_or_default(),
                    message_type,
                    edited: i.edited != 0,
                    timestamp: Date::parse_from_naivetime(i.timestamp),
                    channel_id: ChannelId::new(i.channel_id),
                    author_id,
                })
            }
        }

        Ok(messages)
    }

    async fn get_messages_by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let query = crate::entity::messages::Entity::find()
            .filter(crate::entity::messages::Column::ChannelId.eq(channel_id.id))
            .order_by(
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            )
            .paginate(executor, 50);

        if let Ok(models) = query.fetch().await {
            for i in models {
                let author_id = User::get_user_by_id(i.author_id, executor)
                    .await
                    .ok_or_else(|| "Error Author_id".to_string())?;

                let message_type = MessageType::from_string(i.message_type)
                    .ok_or_else(|| "Error Message_type".to_string())?;

                messages.push(Message {
                    id: i.id,
                    content: i.content.unwrap_or_default(),
                    message_type,
                    edited: i.edited != 0,
                    timestamp: Date::parse_from_naivetime(i.timestamp),
                    channel_id: ChannelId::new(i.channel_id),
                    author_id,
                })
            }
        }

        Ok(messages)
    }

    async fn insert_message(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel {
            id: Set(self.id.clone()),
            content: Set(Some(self.content.clone())),
            message_type: Set(self.message_type.to_string()),
            edited: Set(self.edited as i8),
            timestamp: Set(self.timestamp.0.naive_utc()),
            channel_id: Set(self.channel_id.id.clone()),
            author_id: Set(self.author_id.id.id),
        };

        crate::entity::messages::Entity::insert(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    async fn update_message<T: Into<String> + Send>(
        &mut self,
        content: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let content = content.into();
        let active_model = crate::entity::messages::ActiveModel {
            content: Set(Some(content.clone())),
            edited: Set(true as i8),
            ..Default::default()
        };

        crate::entity::messages::Entity::update(active_model)
            .filter(crate::entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.content = content;
        Ok(())
    }

    async fn delete_message(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = crate::entity::messages::Entity::find_by_id(self.id.clone())
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "Can't delete the message")?;

        let active_model: crate::entity::messages::ActiveModel = model.into();
        crate::entity::messages::Entity::delete(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }
}
