use std::convert::TryFrom;

use super::{
    message::SqlMessage,
    server::{SqlServer, SqlServerId},
};
use crate::{entity::channels::Model, impls::user::UserFrom};
use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType, DirectMessage, DirectMessageInner, ParentId},
    messages::Message,
    server::{Channels, ServerId},
    user::{UserId, UserInfo},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(id: &ChannelId, executor: &DatabaseConnection) -> Option<Channel>;
    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<UserInfo>, String>;
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
    async fn get_channel_by_id(id: &ChannelId, executor: &DatabaseConnection) -> Option<Channel> {
        match Model::get_model_by_id(&id.id, executor).await {
            Ok(model) => model.to_channel(),
            _ => None,
        }
    }

    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<UserInfo>, String> {
        // TODO: Check Permission
        match &self.parent_id {
            ParentId::DirectMessage(directmessage) => match &directmessage.users {
                DirectMessageInner::Users(users) => Ok(users
                    .iter()
                    .map(|user| user.to_userinfo())
                    .collect::<Vec<UserInfo>>()),

                DirectMessageInner::UsersId(usersid) => {
                    let mut vec = Vec::new();

                    for i in usersid {
                        if let Some(user) = i.get_user(executor).await {
                            vec.push(user.to_userinfo());
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
        }
    }

    async fn get_channels_by_server_id(
        server_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String> {
        let parentid = ParentId::ServerId(server_id.clone()).to_string()?;
        let mut channels: Vec<Channel> = Vec::new();

        let models = crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::ParentId.eq(parentid))
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

        crate::entity::channels::Entity::insert(active_channel)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let model = Model::get_model_by_id(&self.id.id, executor)
            .await
            .map_err(|_| "Can't update name".to_string())?;

        let mut active_model: crate::entity::channels::ActiveModel = model.into();
        active_model.name = Set(name.clone());

        crate::entity::channels::Entity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

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
            .map_err(|_| "Can't update description".to_string())?;

        let mut active_model: crate::entity::channels::ActiveModel = model.into();
        active_model.description = Set(Some(description.clone()));

        crate::entity::channels::Entity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.description = description;
        Ok(())
    }

    async fn delete_channel(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model: crate::entity::channels::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor)
                .await
                .map_err(|_| "Can't update name".to_string())?
                .into();

        crate::entity::channels::Entity::delete(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    async fn get_messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String> {
        Message::get_messages_by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn get_channel(&self, executor: &DatabaseConnection) -> Option<Channel>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn get_channel(&self, executor: &DatabaseConnection) -> Option<Channel> {
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

#[async_trait::async_trait]
impl SqlDirectMessages for DirectMessage {
    async fn get_by_userid(
        executor: &DatabaseConnection,
        userid: UserId,
    ) -> Result<Vec<Channel>, String> {
        let user = userid.to_string()?;
        let vec_model = crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::ParentId.contains(&user))
            .all(executor)
            .await
            .map_err(|f| f.to_string())?;

        let mut result = Vec::new();

        for model in vec_model {
            if let Some(channel) = model.to_channel() {
                result.push(channel);
            }
        }

        Ok(result)
    }

    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut dm = self.clone();
        dm.userid_to_user(executor).await?;

        // Name of participant with a coma
        let mut name_of_dm = String::new();
        if let DirectMessageInner::Users(users) = dm.users {
            for (n, i) in users.iter().enumerate() {
                if n == 0 {
                    name_of_dm.push_str(&i.name);
                }

                name_of_dm.push_str(format!(", {}", i.name).as_str());
            }
        }

        let channel = Channel::new_with_parentid(
            name_of_dm,
            String::new(),
            ParentId::DirectMessage(self.clone()),
            ChannelType::DirectMessage,
        )?;
        channel.insert(executor).await
    }
    async fn userid_to_user(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut users = Vec::new();

        match &mut self.users {
            DirectMessageInner::Users(_) => {}
            DirectMessageInner::UsersId(userids) => {
                for userid in userids {
                    let user = userid
                        .get_user(executor)
                        .await
                        .ok_or_else(|| "User not exists".to_string())?;

                    users.push(user);
                }

                self.users = DirectMessageInner::Users(users);
            }
        }

        Ok(())
    }
}
