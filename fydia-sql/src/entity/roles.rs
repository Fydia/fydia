//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use fydia_struct::{
    permission::Permission,
    roles::{ChannelAccess, Role},
    server::ServerId,
};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub serverid: String,
    pub name: String,
    pub color: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub channel_access: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub permission: Option<String>,
}

impl Model {
    pub fn to_role(&self) -> Role {
        Role {
            id: self.id,
            server_id: ServerId::new(self.serverid.clone()),
            name: self.name.clone(),
            color: self.color.clone(),
            channel_access: serde_json::from_str::<ChannelAccess>(
                &self.channel_access.clone().unwrap_or_default(),
            )
            .unwrap_or_default(),
            permission: Permission::from_string(self.permission.clone().unwrap_or_default()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::server::Entity",
        from = "Column::Serverid",
        to = "super::server::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Server,
}

impl Related<super::server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Server.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
