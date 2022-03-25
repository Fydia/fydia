use std::convert::TryFrom;

use super::{
    delete, get_all, get_one, insert,
    message::SqlMessage,
    server::{SqlServer, SqlServerId},
    update,
};
use crate::{entity::channels::Model, impls::user::UserFrom};
use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType, DirectMessage, DirectMessageInner, ParentId},
    messages::Message,
    server::{Channels, ServerId},
    user::UserId,
};
use sea_orm::{ColumnTrait, DatabaseConnection, Set};

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
        let model = get_one(
            crate::entity::channels::Entity,
            vec![crate::entity::channels::Column::Id.eq(id.id.as_str())],
            executor,
        )
        .await?;

        model
            .to_channel()
            .ok_or_else(|| "Cannot convert model to channel".to_string())
    }

    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<UserId>, String> {
        // TODO: Check Permission
        match &self.parent_id {
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
        }
    }

    async fn get_channels_by_server_id(
        server_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String> {
        let parentid = ParentId::ServerId(server_id.clone()).to_string()?;

        let models = get_all(
            crate::entity::channels::Entity,
            vec![crate::entity::channels::Column::ParentId.eq(parentid)],
            executor,
        )
        .await?;

        Ok(Channels(
            models
                .iter()
                .map(|model| model.to_channel())
                .flatten() // Unwrap all model
                .collect::<Vec<Channel>>(),
        ))
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
            .map_err(|_| "Can't update name".to_string())?;

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
            .map_err(|_| "Can't update description".to_string())?;

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
                .map_err(|_| "Can't update name".to_string())?
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

#[async_trait::async_trait]
impl SqlDirectMessages for DirectMessage {
    async fn get_by_userid(
        executor: &DatabaseConnection,
        userid: UserId,
    ) -> Result<Vec<Channel>, String> {
        let user = userid.to_string()?;
        Ok(get_all(
            crate::entity::channels::Entity,
            vec![crate::entity::channels::Column::ParentId.contains(&user)],
            executor,
        )
        .await?
        .iter()
        .map(|model| model.to_channel())
        .flatten()
        .collect())
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
