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
    response::{FydiaResponse, IntoFydia, MapError},
    server::{Channels, ServerId},
    user::UserId,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, Set};
use shared::sea_orm;
#[async_trait::async_trait]
pub trait SqlChannel {
    async fn by_id(id: &ChannelId, executor: &DatabaseConnection)
        -> Result<Channel, FydiaResponse>;
    async fn by_serverid(
        id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, FydiaResponse>;
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<UserId>, FydiaResponse>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn update_description<'a, T: Into<String> + Send>(
        &mut self,
        description: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn by_id(
        id: &ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Channel, FydiaResponse> {
        match Model::get_model_by_id(&id.id, executor).await {
            Ok(model) => model.to_struct(executor).await,
            _ => Err("This Channel doesn't exists".into_error()),
        }
    }
    async fn by_serverid(
        id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, FydiaResponse> {
        let mut channels: Vec<Channel> = Vec::new();
        let models = Model::get_models_by(
            &[entity::channels::Column::ServerId.eq(id.id.as_str())],
            executor,
        )
        .await?;

        for model in models {
            if let Ok(channel) = model.to_struct(executor).await {
                channels.push(channel);
            }
        }

        Ok(Channels(channels))
    }

    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<UserId>, FydiaResponse> {
        // TODO: Check Permission
        let server = self.parent_id.get(executor).await?;
        let members = server.users(executor).await?;

        Ok(members.members)
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let active_channel =
            entity::channels::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        insert(active_channel, executor).await.map(|_| ())
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let name = name.into();
        let model = Model::get_model_by_id(&self.id.id, executor)
            .await
            .map_err(|error| {
                error!("{}", error.get_string());
                "Can't update name".into_error()
            })?;

        let mut active_model: entity::channels::ActiveModel = model.clone().into();
        active_model.name = Set(name.clone());

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update_description<'a, T: Into<String> + Send>(
        &mut self,
        description: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let description = description.into();

        let model = Model::get_model_by_id(&self.id.id, executor)
            .await
            .map_err(|error| {
                error!("{}", error.get_string());
                "Can't update description".into_error()
            })?;

        let mut active_model: entity::channels::ActiveModel = model.clone().into();
        active_model.description = Set(Some(description.clone()));

        update(active_model, executor).await?;

        self.description = description;
        Ok(())
    }

    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let active_model: entity::channels::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor)
                .await
                .map_err(|error| {
                    error!("{}", error.get_string());
                    "Can't find this channel".into_error()
                })?
                .into();

        delete(active_model, executor).await?;

        drop(self);

        Ok(())
    }

    async fn messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, FydiaResponse> {
        Message::by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn channel(&self, executor: &DatabaseConnection) -> Result<Channel, FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn channel(&self, executor: &DatabaseConnection) -> Result<Channel, FydiaResponse> {
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
