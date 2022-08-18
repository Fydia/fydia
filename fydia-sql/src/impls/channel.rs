use std::convert::TryFrom;

use super::{
    basic_model::BasicModel,
    delete, insert,
    message::SqlMessage,
    server::{SqlServer, SqlServerId},
    update,
};

use entity::channels::Model;
use fydia_struct::{
    channel::{Channel, ChannelId},
    messages::Message,
    user::UserId,
};
use fydia_utils::async_trait;
use sea_orm::{DatabaseConnection, Set};

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn by_id(id: &ChannelId, executor: &DatabaseConnection) -> Result<Channel, String>;
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<UserId>, String>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_description<T: Into<String> + Send>(
        &mut self,
        description: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn by_id(id: &ChannelId, executor: &DatabaseConnection) -> Result<Channel, String> {
        match Model::get_model_by_id(&id.id, executor).await {
            Ok(model) => model.to_struct(executor).await,
            _ => Err("This Channel doesn't exists".to_string()),
        }
    }

    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<UserId>, String> {
        // TODO: Check Permission
        let server = self.parent_id.get(executor).await?;
        let members = server.users(executor).await?;

        Ok(members.members)
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = entity::channels::ActiveModel::try_from(self.clone())?;

        insert(active_channel, executor).await
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let model = Model::get_model_by_id(&self.id.id, executor)
            .await
            .map_err(|error| {
                error!("{error}");
                "Can't update name".to_string()
            })?;

        let mut active_model: entity::channels::ActiveModel = model.clone().into();
        active_model.name = Set(name.clone());

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update_description<T: Into<String> + Send>(
        &mut self,
        description: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let description = description.into();

        let model = Model::get_model_by_id(&self.id.id, executor)
            .await
            .map_err(|error| {
                error!("{error}");
                "Can't update description".to_string()
            })?;

        let mut active_model: entity::channels::ActiveModel = model.clone().into();
        active_model.description = Set(Some(description.clone()));

        update(active_model, executor).await?;

        self.description = description;
        Ok(())
    }

    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model: entity::channels::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor)
                .await
                .map_err(|error| {
                    error!("{error}");
                    "Can't find this channel".to_string()
                })?
                .into();

        delete(active_model, executor).await?;

        self = Self::default();

        Ok(())
    }

    async fn messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String> {
        Message::by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn channel(&self, executor: &DatabaseConnection) -> Result<Channel, String>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn channel(&self, executor: &DatabaseConnection) -> Result<Channel, String> {
        Channel::by_id(self, executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlDirectMessages {
    async fn by_userid(
        executor: &DatabaseConnection,
        userid: UserId,
    ) -> Result<Vec<Channel>, String>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn userid_to_user(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
}
