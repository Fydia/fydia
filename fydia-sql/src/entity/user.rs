//! SeaORM Entity. Generated by sea-orm-codegen 0.2.3

use fydia_struct::{
    instance::Instance,
    server::Servers,
    user::{User, UserId},
};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "User")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub instance: Option<String>,
    pub token: String,
    #[sea_orm(column_type = "Text")]
    pub email: String,
    #[sea_orm(column_type = "Text")]
    pub password: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub server: Option<String>,
}

impl Model {
    pub fn to_user(&self) -> Option<User> {
        let servers = self
            .server
            .as_ref()
            .map(|server| Servers(serde_json::from_str(server.as_str()).unwrap_or_default()))?;

        Some(User {
            id: UserId::new(self.id),
            name: self.name.clone(),
            description: self.description.clone(),
            email: self.email.clone(),
            instance: Instance::default(),
            token: Some(self.token.clone()),
            password: Some(self.password.clone()),
            servers,
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::messages::Entity")]
    Messages,
    #[sea_orm(has_many = "super::server::Entity")]
    Server,
}

impl Related<super::messages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Server.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
