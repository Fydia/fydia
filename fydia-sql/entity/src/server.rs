//! `SeoORM` Entity. Generated by sea-orm-codegen 0.6.0

use std::convert::TryFrom;

use fydia_struct::server::{Server, ServerError};
use sea_orm::{entity::prelude::*, Set};
use shared::sea_orm;
//use crate::impls::{channel::SqlChannel, members::SqlMembers, role::SqlRoles};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "server")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub owner: u32,
    #[sea_orm(column_type = "Text", nullable)]
    pub icon: Option<String>,
}

impl TryFrom<Server> for ActiveModel {
    type Error = ServerError;

    fn try_from(value: Server) -> Result<Self, Self::Error> {
        Ok(ActiveModel {
            id: Set(value.id.id.clone()),
            name: Set(value.name.clone()),
            owner: Set(value.owner.0.get_id()?),
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
