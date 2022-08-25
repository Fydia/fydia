#![allow(irrefutable_let_patterns)]

use std::convert::TryFrom;

use fydia_struct::{channel::ChannelId, messages::Message, response::FydiaResponse};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use super::{basic_model::BasicModel, delete, insert};
use entity::messages::Model;
use fydia_utils::async_trait;

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn by_userid<'a>(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse<'a>>;
    async fn by_channel<'a>(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse<'a>>;
    async fn by_id<'a>(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, FydiaResponse<'a>>;
    async fn insert<'a>(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
    async fn update<'a>(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>>;
    async fn delete<'a>(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn by_userid<'a>(
        id: i32,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse<'a>> {
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

    async fn by_channel<'a>(
        channel_id: ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Message>, FydiaResponse<'a>> {
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

    async fn by_id<'a>(
        message_id: &str,
        executor: &DatabaseConnection,
    ) -> Result<Message, FydiaResponse<'a>> {
        let message = Model::get_model_by_id(message_id, executor).await?;

        message.to_struct(executor).await
    }

    async fn insert<'a>(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        let active_model = entity::messages::ActiveModel::try_from(self.clone())
            .map_err(FydiaResponse::StringError)?;

        insert(active_model, executor).await.map(|_| ())
    }

    async fn update<'a>(
        &mut self,
        content: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>> {
        let model = entity::messages::ActiveModel::try_from(self.clone())
            .map_err(FydiaResponse::StringError)?;

        entity::messages::Entity::update(model)
            .filter(entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
            .map_err(|f| FydiaResponse::StringError(f.to_string()))?;

        self.content = content.to_string();

        Ok(())
    }

    async fn delete<'a>(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        let model = Model::get_model_by_id(&self.id, executor).await?;
        let active_model: entity::messages::ActiveModel = model.clone().into();
        delete(active_model, executor).await?;

        drop(self);

        Ok(())
    }
}
