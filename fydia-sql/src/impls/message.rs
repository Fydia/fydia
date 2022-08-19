#![allow(irrefutable_let_patterns)]

use std::convert::TryFrom;

use fydia_struct::{channel::ChannelId, messages::Message};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use super::{basic_model::BasicModel, delete, insert};
use entity::messages::Model;
use fydia_utils::async_trait;

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn by_userid(id: i32, executor: &DatabaseConnection) -> Result<Vec<Message>, String>;
    async fn by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String>;
    async fn by_id(message_id: &str, executor: &DatabaseConnection) -> Result<Message, String>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update(&mut self, content: &str, executor: &DatabaseConnection) -> Result<(), String>;
    async fn delete(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn by_userid(id: i32, executor: &DatabaseConnection) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let mut query = entity::messages::Entity::find()
            .filter(entity::messages::Column::AuthorId.eq(id))
            .order_by(entity::messages::Column::Timestamp, sea_orm::Order::Asc)
            .paginate(executor, 50);

        while let Ok(Some(e)) = query.fetch_and_next().await {
            for i in e {
                if let Ok(message) = i.to_struct(executor).await {
                    messages.push(message);
                }
            }
        }

        Ok(messages)
    }

    async fn by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let query = entity::messages::Entity::find()
            .filter(entity::messages::Column::ChannelId.eq(channel_id.id))
            .order_by(entity::messages::Column::Timestamp, sea_orm::Order::Asc)
            .paginate(executor, 50);

        if let Ok(models) = query.fetch().await {
            for i in models {
                if let Ok(message) = i.to_struct(executor).await {
                    messages.push(message);
                }
            }
        }

        Ok(messages)
    }

    async fn by_id(message_id: &str, executor: &DatabaseConnection) -> Result<Message, String> {
        let message = Model::get_model_by_id(message_id, executor).await?;

        message.to_struct(executor).await
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model = entity::messages::ActiveModel::try_from(self.clone())?;

        insert(active_model, executor).await
    }

    async fn update(&mut self, content: &str, executor: &DatabaseConnection) -> Result<(), String> {
        let model = entity::messages::ActiveModel::try_from(self.clone())?;

        entity::messages::Entity::update(model)
            .filter(entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.content = content.to_string();

        Ok(())
    }

    async fn delete(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = Model::get_model_by_id(&self.id, executor).await?;
        let active_model: entity::messages::ActiveModel = model.clone().into();
        delete(active_model, executor).await?;

        drop(self);

        Ok(())
    }
}
