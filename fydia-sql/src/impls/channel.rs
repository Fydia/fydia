use super::{
    message::SqlMessage,
    server::{SqlServer, SqlServerId},
};
use crate::impls::user::UserIdSql;
use fydia_struct::{
    channel::{Channel, ChannelId, ChannelType, DirectMessage, DirectMessageValue, ParentId},
    messages::Message,
    server::{Channels, ServerId},
    user::{User, UserId},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(id: ChannelId, executor: &DatabaseConnection) -> Option<Channel>;
    async fn get_user_of_channel(&self, executor: &DatabaseConnection)
        -> Result<Vec<User>, String>;
    async fn get_channels_by_server_id(
        server_id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String>;
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_description(
        &mut self,
        description: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn delete_channel(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn get_messages(&self, executor: &DatabaseConnection) -> Result<Vec<Message>, String>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn get_channel_by_id(id: ChannelId, executor: &DatabaseConnection) -> Option<Channel> {
        match crate::entity::channels::Entity::find_by_id(id.id)
            .one(executor)
            .await
        {
            Ok(Some(model)) => model.to_channel(),
            _ => None,
        }
    }

    async fn get_user_of_channel(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<Vec<User>, String> {
        // TODO: Check Permission
        match &self.parent_id {
            ParentId::DirectMessage(directmessage) => match &directmessage.users {
                DirectMessageValue::Users(users) => {
                    return Ok(users.clone());
                }
                DirectMessageValue::UsersId(usersid) => {
                    let mut vec = Vec::new();
                    for i in usersid {
                        if let Some(user) = i.get_user(executor).await {
                            vec.push(user);
                        }
                    }

                    return Ok(vec);
                }
            },
            ParentId::ServerId(serverid) => {
                if let Ok(server) = serverid.get_server(executor).await {
                    if let Ok(members) = server.get_user(executor).await {
                        return Ok(members.members);
                    }
                }
            }
        };

        Ok(Vec::new())
    }

    async fn get_channels_by_server_id(
        server_id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Channels, String> {
        let parentid = ParentId::ServerId(server_id).to_string()?;
        let mut channels: Vec<Channel> = Vec::new();
        match crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::ParentId.eq(parentid))
            .all(executor)
            .await
        {
            Ok(models) => {
                for model in models {
                    if let Some(channel) = model.to_channel() {
                        channels.push(channel);
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        }

        Ok(Channels(channels))
    }
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let parent_id = self.parent_id.to_string()?;
        let active_channel = crate::entity::channels::ActiveModel {
            id: Set(self.id.id.clone()),
            parent_id: Set(parent_id),
            name: Set(self.name.clone()),
            description: Set(Some(self.description.clone())),
            channel_type: Set(Some(self.channel_type.to_string())),
        };
        match crate::entity::channels::Entity::insert(active_channel)
            .exec(executor)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.id.clone())
            .one(executor)
            .await
        {
            Ok(Some(e)) => {
                let mut active_model: crate::entity::channels::ActiveModel = e.into();

                active_model.name = Set(name.clone());
                match crate::entity::channels::Entity::update(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => {
                        self.name = name;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error".to_string()),
        }
    }

    async fn update_description(
        &mut self,
        description: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.id.clone())
            .one(executor)
            .await
        {
            Ok(Some(e)) => {
                let mut active_model: crate::entity::channels::ActiveModel = e.into();

                active_model.description = Set(Some(description.clone()));
                match crate::entity::channels::Entity::update(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => {
                        self.description = description;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error".to_string()),
        }
    }

    async fn delete_channel(&self, executor: &DatabaseConnection) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.id.clone())
            .one(executor)
            .await
        {
            Ok(Some(e)) => {
                let active_model: crate::entity::channels::ActiveModel = e.into();
                match crate::entity::channels::Entity::delete(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error".to_string()),
        }
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
        Channel::get_channel_by_id(self.clone(), executor).await
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
        let one = crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::ParentId.contains(&user))
            .all(executor)
            .await;
        let mut r = Vec::new();
        match one {
            Ok(result) => {
                for i in result {
                    if let Some(e) = i.to_channel() {
                        r.push(e);
                    } else {
                        return Err("Not Exists".to_string());
                    }
                }
            }
            Err(e) => return Err(e.to_string()),
        };
        Ok(r)
    }
    async fn insert(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let channel = Channel::new_with_parentid(
            "",
            "",
            ParentId::DirectMessage(self.clone()),
            ChannelType::DirectMessage,
        );
        channel.insert(executor).await
    }
    async fn userid_to_user(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut users = Vec::new();
        match &mut self.users {
            DirectMessageValue::Users(_) => {}
            DirectMessageValue::UsersId(e) => {
                for i in e {
                    match i.get_user(executor).await {
                        Some(e) => users.push(e),
                        None => return Err("User not exists".to_string()),
                    };
                }

                self.users = DirectMessageValue::Users(users);
            }
        }

        Ok(())
    }
}
