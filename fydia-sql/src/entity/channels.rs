use std::convert::TryFrom;

use fydia_struct::channel::{Channel, ChannelId, ChannelType, ParentId};
use sea_orm::{entity::prelude::*, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Channels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub parent_id: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub channel_type: Option<String>,
}

impl Model {
    pub fn to_channel(&self) -> Option<Channel> {
        let channel_type = self.channel_type.as_ref().map(ChannelType::from_string)?;
        let parent_id = serde_json::from_str::<ParentId>(&self.parent_id).ok()?;
        Some(Channel {
            id: ChannelId::new(self.id.clone()),
            name: self.name.clone(),
            parent_id,
            channel_type,
            description: self.description.clone().unwrap_or_default(),
        })
    }

    /// Get model with id
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist with this id
    pub async fn get_model_by_id(id: &str, executor: &DatabaseConnection) -> Result<Self, String> {
        match crate::entity::channels::Entity::find_by_id(id.to_string())
            .one(executor)
            .await
        {
            Ok(Some(model)) => Ok(model),
            _ => Err(String::from("No Model with this id")),
        }
    }
}

impl TryFrom<Channel> for ActiveModel {
    type Error = String;

    fn try_from(channel: Channel) -> Result<Self, Self::Error> {
        let parent_id = serde_json::to_string(&channel.parent_id).map_err(|f| f.to_string())?;

        Ok(Self {
            id: Set(channel.id.id.clone()),
            parent_id: Set(parent_id),
            name: Set(channel.name.clone()),
            description: Set(Some(channel.description.clone())),
            channel_type: Set(Some(channel.channel_type.to_string())),
        })
    }
}

impl TryFrom<&Channel> for ActiveModel {
    type Error = String;

    fn try_from(channel: &Channel) -> Result<Self, Self::Error> {
        Self::try_from(channel.clone())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::messages::Entity")]
    Messages,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
