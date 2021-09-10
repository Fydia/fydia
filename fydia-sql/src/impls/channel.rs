use std::sync::Arc;

use super::message::SqlMessage;
use fydia_struct::{
    channel::{Channel, ChannelId},
    messages::Message,
    server::Channels,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlChannel {
    async fn get_channel_by_id(
        id: ChannelId,
        executor: &Arc<DatabaseConnection>,
    ) -> Option<Channel>;
    async fn get_channels_by_server_id(
        server_id: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Channels, String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
    async fn update_description(
        &mut self,
        description: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
    async fn delete_channel(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String>;
    async fn get_messages(
        &self,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String>;
}

#[async_trait::async_trait]
impl SqlChannel for Channel {
    async fn get_channel_by_id(
        id: ChannelId,
        executor: &Arc<DatabaseConnection>,
    ) -> Option<Channel> {
        match crate::entity::channels::Entity::find_by_id(id.id)
            .one(executor)
            .await
        {
            Ok(Some(model)) => model.to_channel(),
            _ => None,
        }
    }

    async fn get_channels_by_server_id(
        server_id: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Channels, String> {
        let mut server_id = server_id;
        if server_id.len() > 10 {
            server_id = server_id.split_at(10).0.to_string();
        }
        let mut channels: Vec<Channel> = Vec::new();
        match crate::entity::channels::Entity::find()
            .filter(crate::entity::channels::Column::Serverid.eq(server_id))
            .all(executor)
            .await
        {
            Ok(models) => {
                for model in models {
                    match model.to_channel() {
                        Some(channel) => {
                            channels.push(channel);
                        }
                        _ => {}
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

    async fn update_name(
        &mut self,
        name: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.as_str())
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
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.as_str())
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

    async fn delete_channel(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String> {
        match crate::entity::channels::Entity::find_by_id(self.id.as_str())
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

    async fn get_messages(
        &self,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String> {
        Message::get_messages_by_channel(self.id.clone(), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlChannelId {
    async fn get_channel(&self, executor: &Arc<DatabaseConnection>) -> Option<Channel>;
}

#[async_trait::async_trait]
impl SqlChannelId for ChannelId {
    async fn get_channel(&self, executor: &Arc<DatabaseConnection>) -> Option<Channel> {
        Channel::get_channel_by_id(self.clone(), executor).await
    }
}
