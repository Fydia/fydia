//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use std::convert::TryFrom;

use fydia_struct::{
    channel::Channel,
    roles::Role,
    server::{Members, Server, ServerId},
    user::UserId,
};
use sea_orm::{entity::prelude::*, Set};

use crate::impls::{channel::SqlChannel, role::SqlRoles};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Server")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub owner: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub icon: Option<String>,
}

impl Model {
    pub async fn to_server(&self, executor: &DatabaseConnection) -> Result<Server, String> {
        let members = Members::new();
        let roles = Role::get_roles_by_server_id(self.id.clone(), executor).await?;

        let channel =
            Channel::get_channels_by_server_id(&ServerId::new(self.id.clone()), executor).await?;

        Ok(Server {
            id: ServerId::new(self.id.clone()),
            name: self.name.clone(),
            owner: UserId::new(self.owner),
            icon: self.icon.clone().unwrap_or_else(|| "Error".to_string()),
            members,
            channel,
            roles,
            emoji: Vec::new(),
        })
    }

    pub async fn get_model_by_id(id: &str, executor: &DatabaseConnection) -> Result<Model, String> {
        match crate::entity::server::Entity::find_by_id(id.to_string())
            .one(executor)
            .await
        {
            Ok(Some(model)) => Ok(model),
            _ => Err("No Server with this id".to_string()),
        }
    }
}

impl TryFrom<Server> for ActiveModel {
    type Error = String;

    fn try_from(value: Server) -> Result<Self, Self::Error> {
        Ok(crate::entity::server::ActiveModel {
            id: Set(value.id.id.clone()),
            name: Set(value.name.clone()),
            owner: Set(value.owner.0),
            icon: Set(Some(value.icon)),
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Owner",
        to = "super::user::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    User,
    #[sea_orm(has_many = "super::members::Entity")]
    Members,
    #[sea_orm(has_many = "super::roles::Entity")]
    Roles,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
