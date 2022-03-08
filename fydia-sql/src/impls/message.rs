#![allow(irrefutable_let_patterns)]

use std::convert::TryFrom;

use fydia_struct::{channel::ChannelId, messages::Message};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};

use crate::entity::messages::Model;

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
    async fn update_message(
        &mut self,
        content: &str,
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
                if let Ok(message) = i.to_message(executor).await {
                    messages.push(message);
                }
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
                if let Ok(message) = i.to_message(executor).await {
                    messages.push(message);
                }
            }
        }

        Ok(messages)
    }

    async fn insert_message(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel::try_from(self.clone())?;

        crate::entity::messages::Entity::insert(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    async fn update_message(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel {
            content: Set(Some(content.to_string())),
            edited: Set(true as i8),
            ..Default::default()
        };

        crate::entity::messages::Entity::update(active_model)
            .filter(crate::entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.content = content.to_string();

        Ok(())
    }

    async fn delete_message(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = Model::get_model_by_id(&self.id, executor).await?;
        let active_model: crate::entity::messages::ActiveModel = model.into();
        crate::entity::messages::Entity::delete(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }
}
