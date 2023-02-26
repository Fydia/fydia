#![allow(irrefutable_let_patterns)]

use std::convert::TryFrom;

use super::{basic_model::BasicModel, delete, insert};
use entity::messages::Model;
use fydia_struct::{
    channel::ChannelId,
    messages::Message,
    response::{FydiaResponse, MapError},
};
use fydia_utils::async_trait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use shared::sea_orm;

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn by_userid(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse>;
    async fn by_channel(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse>;
    async fn by_id(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, FydiaResponse>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn update(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn by_userid(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse> {
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
    ) -> Result<Vec<Message>, FydiaResponse> {
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

    async fn by_id(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, FydiaResponse> {
        let message = Model::get_model_by_id(message_id, executor).await?;

        message.to_struct(executor).await
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let active_model =
            entity::messages::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        insert(active_model, executor).await.map(|_| ())
    }

    async fn update(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let model =
            entity::messages::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        entity::messages::Entity::update(model)
            .filter(entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
            .error_to_fydiaresponse()?;

        self.content = content.to_string();

        Ok(())
    }

    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let model = Model::get_model_by_id(&self.id, executor).await?;
        let active_model: entity::messages::ActiveModel = model.clone().into();
        delete(active_model, executor).await?;

        drop(self);

        Ok(())
    }
}
