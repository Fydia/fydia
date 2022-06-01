use std::convert::TryFrom;

use super::{
    delete, insert,
    message::SqlMessage,
    server::{SqlServer, SqlServerId},
    update,
};
use crate::entity::channels::Model;
use fydia_struct::{
    channel::{Channel, ChannelId},
    messages::Message,
    server::{Channels, ServerId},
    user::UserId,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(
        id: &ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Channel, String>;
    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<UserId>, String>;
    async fn get_channels_by_server_id(
        server_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String>;
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
    async fn delete_channel(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn get_messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn get_channel_by_id(
        id: &ChannelId,
        executor: &DatabaseConnection,
    ) -> Result<Channel, String> {
        match Model::get_model_by_id(&id.id, executor).await {
            Ok(model) => model
                .to_channel()
                .ok_or_else(|| "Cannot convert model to channel".to_string()),
            _ => Err("This Channel doesn't exists".to_string()),
        }
    }

    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<UserId>, String> {
        // TODO: Check Permission
        /* match &self.parent_id {
            ParentId::DirectMessage(directmessage) => match &directmessage.users {
                DirectMessageInner::Users(users) => Ok(users
                    .iter()
                    .map(|user| user.id.clone())
                    .collect::<Vec<UserId>>()),

                DirectMessageInner::UsersId(usersid) => {
                    let mut vec = Vec::new();

                    for i in usersid {
                        if let Some(user) = i.get_user(executor).await {
                            vec.push(user.id);
                        }
                    }

                    Ok(vec)
                }
            },
            ParentId::ServerId(serverid) => {
                let server = serverid.get_server(executor).await?;
                let members = server.get_user(executor).await?;

                Ok(members.members)
            }
        };*/

        let server = self.parent_id.get_server(executor).await?;
        let members = server.get_user(executor).await?;

        Ok(members.members)
    }

    async fn get_channels_by_server_id(
        server_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String> {
        let mut channels: Vec<Channel> = Vec::new();
        let models = crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::ParentId.eq(server_id.id.as_str()))
            .all(executor)
            .await
            .map_err(|f| f.to_string())?;

        for model in models {
            if let Some(channel) = model.to_channel() {
                channels.push(channel);
            }
        }

        Ok(Channels(channels))
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = crate::entity::channels::ActiveModel::try_from(self.clone())?;

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

        let mut active_model: crate::entity::channels::ActiveModel = model.into();
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

        let mut active_model: crate::entity::channels::ActiveModel = model.into();
        active_model.description = Set(Some(description.clone()));

        update(active_model, executor).await?;

        self.description = description;
        Ok(())
    }

    async fn delete_channel(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model: crate::entity::channels::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor)
                .await
                .map_err(|error| {
                    error!("{error}");
                    "Can't find this channel".to_string()
                })?
                .into();

        delete(active_model, executor).await
    }

    async fn get_messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String> {
        Message::get_messages_by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn get_channel(&self, executor: &DatabaseConnection) -> Result<Channel, String>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn get_channel(&self, executor: &DatabaseConnection) -> Result<Channel, String> {
        Channel::get_channel_by_id(self, executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlDirectMessages {
    async fn get_by_userid(
        executor: &DatabaseConnection,
        userid: UserId,
    ) -> Result<Vec<Channel>, String>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn userid_to_user(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
}
