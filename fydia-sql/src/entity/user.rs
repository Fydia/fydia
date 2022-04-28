//! `SeaORM` Entity. Generated by sea-orm-codegen 0.2.3

use std::convert::TryFrom;

use fydia_struct::{
    instance::Instance,
    server::{Members, Servers},
    user::{User, UserId},
};
use sea_orm::{entity::prelude::*, sea_query::IntoCondition, Set};

use crate::impls::members::SqlMembers;

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
}

impl Model {
    /// Return model from user
    ///
    /// # Errors
    /// Return an error if:
    /// * Cannot get servers of user
    pub async fn to_user(&self, executor: &DatabaseConnection) -> Result<User, String> {
        let servers =
            Servers(Members::get_servers_by_usersid(&UserId::new(self.id), executor).await?);

        Ok(User {
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

    /// Get model with id of model
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Table doesn't exist
    /// * id isn't in table
    pub async fn get_model_by_id(id: i32, executor: &DatabaseConnection) -> Result<Model, String> {
        Self::get_model_by(crate::entity::user::Column::Id.eq(id), executor).await
    }

    /// Get model with a token
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Table doesn't exist
    /// * Token isn't in table
    pub async fn get_model_by_token(
        token: &str,
        executor: &DatabaseConnection,
    ) -> Result<Model, String> {
        Self::get_model_by(crate::entity::user::Column::Token.contains(token), executor).await
    }

    /// Get model with any condition
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Table doesn't exist
    /// * Token isn't in table
    pub async fn get_model_by<F>(
        condition: F,
        executor: &DatabaseConnection,
    ) -> Result<Model, String>
    where
        F: IntoCondition,
    {
        match crate::entity::user::Entity::find()
            .filter(condition)
            .one(executor)
            .await
        {
            Ok(Some(model)) => Ok(model),
            Err(error) => Err(format!("{error}")),
            _ => Err("No model with this condition".to_string()),
        }
    }
}

impl TryFrom<User> for ActiveModel {
    type Error = String;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        let instance_json = serde_json::to_string(&value.instance).map_err(|f| f.to_string())?;
        let password = value
            .password
            .clone()
            .ok_or_else(|| "Password is empty".to_string())?;

        Ok(Self {
            name: Set(value.name.clone()),
            token: Set(value.token.unwrap_or_default()),
            email: Set(value.email.clone()),
            password: Set(password),
            instance: Set(Some(instance_json)),
            description: Set(value.description),
            ..Default::default()
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::members::Entity")]
    Members,
    #[sea_orm(has_many = "super::messages::Entity")]
    Messages,
    #[sea_orm(has_many = "super::server::Entity")]
    Server,
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
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
