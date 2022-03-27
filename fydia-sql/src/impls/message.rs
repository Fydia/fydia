#![allow(irrefutable_let_patterns)]

use std::convert::TryFrom;

use fydia_struct::{channel::ChannelId, messages::Message};
use sea_orm::{ColumnTrait, DatabaseConnection};

use crate::entity::messages::Model;

use super::{delete, get_all_with_limit_with_order, insert, update};

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
    async fn get_message_by_id(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, String>;
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
        let models = get_all_with_limit_with_order(
            crate::entity::messages::Entity,
            vec![crate::entity::messages::Column::AuthorId.eq(id)],
            (
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            ),
            50,
            executor,
        )
        .await?;

        let mut messages = Vec::new();
        for i in models {
            messages.push(i.to_message(executor).await?);
        }

        Ok(messages)
    }

    async fn get_messages_by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, String> {
        let models = get_all_with_limit_with_order(
            crate::entity::messages::Entity,
            vec![crate::entity::messages::Column::ChannelId.eq(channel_id.id)],
            (
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            ),
            50,
            executor,
        )
        .await?;

        let mut messages = Vec::new();
        for i in models {
            if let Ok(message) = i.to_message(executor).await {
                messages.push(message);
            }
        }

        Ok(messages)
    }

    async fn get_message_by_id(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, String> {
        let message = Model::get_model_by_id(message_id, executor).await?;

        message.to_message(executor).await
    }

    async fn insert_message(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel::try_from(self.clone())?;

        insert(active_model, executor).await
    }

    async fn update_message(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let model = crate::entity::messages::ActiveModel::try_from(self.clone())?;

        update(model, executor).await?;

        self.content = content.to_string();

        Ok(())
    }

    async fn delete_message(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = Model::get_model_by_id(&self.id, executor).await?;
        let active_model: crate::entity::messages::ActiveModel = model.into();
        delete(active_model, executor).await?;

        // Poisoning struct to not be used after
        *self = Message::default();

        Ok(())
    }
}
