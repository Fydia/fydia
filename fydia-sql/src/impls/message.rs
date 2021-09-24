#![allow(irrefutable_let_patterns)]

use fydia_struct::{
    channel::ChannelId,
    messages::{Message, MessageType, SqlDate},
    user::User,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set};
use std::sync::Arc;

use super::user::SqlUser;

#[async_trait::async_trait]
pub trait SqlMessage {
    async fn get_messages_by_user_id(
        id: i32,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String>;
    async fn get_messages_by_channel(
        channel_id: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String>;
    async fn insert_message(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String>;
    async fn update_message(
        &mut self,
        content: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
    async fn delete_message(&mut self, executor: &Arc<DatabaseConnection>) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlMessage for Message {
    async fn get_messages_by_user_id(
        id: i32,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let mut query = crate::entity::messages::Entity::find()
            .filter(crate::entity::messages::Column::AuthorId.eq(id))
            .order_by(
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            )
            .paginate(executor, 50);
        while let Ok(Some(e)) = query.fetch_and_next().await {
            for i in e {
                let author_id = match User::get_user_by_id(i.author_id, executor).await {
                    Some(author_id) => author_id,
                    None => return Err("Error Author_id".to_string()),
                };

                let message_type = match MessageType::from_string(i.message_type) {
                    Some(e) => e,
                    None => return Err("Error Message_type".to_string()),
                };

                messages.push(Message {
                    id: i.id,
                    content: i.content.unwrap_or_default(),
                    message_type,
                    edited: i.edited != 0,
                    timestamp: SqlDate::parse_from_naivetime(i.timestamp),
                    channel_id: ChannelId::new(i.channel_id),
                    author_id,
                })
            }
        }

        Ok(messages)
    }

    async fn get_messages_by_channel(
        channel_id: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Vec<Message>, String> {
        let mut messages = Vec::new();
        let mut query = crate::entity::messages::Entity::find()
            .filter(crate::entity::messages::Column::ChannelId.eq(channel_id))
            .order_by(
                crate::entity::messages::Column::Timestamp,
                sea_orm::Order::Asc,
            )
            .paginate(executor, 50);
        let mut n = 0;
        while let e = query.fetch_and_next().await {
            match e {
                Ok(e) => match e {
                    Some(vec) => {
                        for i in vec {
                            let author_id = match User::get_user_by_id(i.author_id, executor).await
                            {
                                Some(author_id) => author_id,
                                None => return Err("Error Author_id".to_string()),
                            };

                            let message_type = match MessageType::from_string(i.message_type) {
                                Some(e) => e,
                                None => return Err("Error Message_type".to_string()),
                            };

                            messages.push(Message {
                                id: i.id,
                                content: i.content.unwrap_or_default(),
                                message_type,
                                edited: i.edited != 0,
                                timestamp: SqlDate::parse_from_naivetime(i.timestamp),
                                channel_id: ChannelId::new(i.channel_id),
                                author_id,
                            })
                        }
                    }
                    None => {
                        return Ok(messages);
                    }
                },
                Err(e) => {
                    error!(e.to_string());
                    return Err(e.to_string());
                }
            }
            if n == 50 {
                break;
            }
            n += 1;
        }

        Ok(messages)
    }

    async fn insert_message(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel {
            id: Set(self.id.clone()),
            content: Set(Some(self.content.clone())),
            message_type: Set(self.message_type.to_string()),
            edited: Set(self.edited as i8),
            timestamp: Set(self.timestamp.0.naive_utc()),
            channel_id: Set(self.channel_id.id.clone()),
            author_id: Set(self.author_id.id),
        };

        match crate::entity::messages::Entity::insert(active_model)
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

    async fn update_message(
        &mut self,
        content: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        let active_model = crate::entity::messages::ActiveModel {
            content: Set(Some(content.to_string())),
            edited: Set(true as i8),
            ..Default::default()
        };

        match crate::entity::messages::Entity::update(active_model)
            .filter(crate::entity::messages::Column::Id.eq(self.id.as_str()))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.content = content;
                return Ok(());
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        }
    }

    async fn delete_message(&mut self, executor: &Arc<DatabaseConnection>) -> Result<(), String> {
        match crate::entity::messages::Entity::find_by_id(self.id.clone())
            .one(executor)
            .await
        {
            Ok(Some(model)) => {
                let active_model: crate::entity::messages::ActiveModel = model.into();
                match crate::entity::messages::Entity::delete(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => return Ok(()),
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
            _ => return Err("Cannot get database error".to_string()),
        }
    }
}
