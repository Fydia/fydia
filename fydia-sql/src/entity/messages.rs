//! `SeaORM` Entity

use std::convert::TryFrom;

use fydia_struct::{
    channel::ChannelId,
    messages::{Date, Message, MessageType},
    user::User,
};
use sea_orm::{entity::prelude::*, Set};

use crate::impls::user::SqlUser;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub content: Option<String>,
    pub message_type: String,
    pub edited: i8,
    pub timestamp: DateTime,
    pub channel_id: String,
    pub author_id: i32,
}

impl Model {
    /// Convert model to message
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist
    /// * Cannot get User
    /// * Cannot deserialize `message_type`
    pub async fn to_message(&self, executor: &DatabaseConnection) -> Result<Message, String> {
        let author_id = User::get_user_by_id(self.author_id, executor)
            .await
            .ok_or_else(|| "Error Author_Id".to_string())?
            .to_userinfo();

        let message_type = MessageType::from_string(&self.message_type)
            .ok_or_else(|| "Error Message_type".to_string())?;

        Ok(Message {
            id: self.id.clone(),
            content: self.content.clone().unwrap_or_default(),
            message_type,
            edited: self.edited != 0,
            timestamp: Date::parse_from_naivetime(self.timestamp),
            channel_id: ChannelId::new(self.channel_id.clone()),
            author_id,
        })
    }

    /// Get model by id
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist
    pub async fn get_model_by_id(id: &str, executor: &DatabaseConnection) -> Result<Self, String> {
        match crate::entity::messages::Entity::find_by_id(id.to_string())
            .one(executor)
            .await
        {
            Ok(Some(model)) => Ok(model),
            _ => Err("No Message with this id".to_string()),
        }
    }
}

impl TryFrom<Message> for ActiveModel {
    type Error = String;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Set(value.id.clone()),
            content: Set(Some(value.content)),
            message_type: Set(value.message_type.to_string()),
            timestamp: Set(value.timestamp.0.naive_utc()),
            edited: Set(value.edited as i8),
            channel_id: Set(value.channel_id.id.clone()),
            author_id: Set(value.author_id.id.0),
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::channels::Entity",
        from = "Column::ChannelId",
        to = "super::channels::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Channels,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    User,
}

impl Related<super::channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
