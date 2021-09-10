//! SeaORM Entity. Generated by sea-orm-codegen 0.2.0

use fydia_struct::{
    channel::{Channel, ChannelType},
    server::ServerId,
};
use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "Channels"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: String,
    pub serverid: String,
    pub name: String,
    pub description: Option<String>,
    pub channel_type: Option<String>,
}

impl Model {
    pub fn from_channel(channel: Channel) -> Self {
        Self {
            id: channel.id,
            serverid: channel.server_id.short_id,
            name: channel.name,
            description: Some(channel.description),
            channel_type: Some(channel.channel_type.to_string()),
        }
    }
    pub fn to_channel(&self) -> Option<Channel> {
        let channel_type = match self.channel_type.clone() {
            Some(e) => ChannelType::from_string(e),
            None => return None,
        };
        Some(Channel {
            id: self.id.clone(),
            name: self.name.clone(),
            server_id: ServerId::new(self.serverid.clone()),
            channel_type,
            description: self.description.clone().unwrap_or_default(),
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Serverid,
    Name,
    Description,
    ChannelType,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = String;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Messages,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::String(Some(15u32)).def(),
            Self::Serverid => ColumnType::String(Some(10u32)).def(),
            Self::Name => ColumnType::Text.def(),
            Self::Description => ColumnType::Text.def().null(),
            Self::ChannelType => ColumnType::String(Some(100u32)).def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Messages => Entity::has_many(super::messages::Entity).into(),
        }
    }
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
