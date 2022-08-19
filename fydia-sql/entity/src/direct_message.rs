//! `SeoORM` Entity. Generated by sea-orm-codegen 0.6.0

use std::convert::TryFrom;

use fydia_struct::{directmessage::DirectMessage, utils::Id};
use sea_orm::{
    entity::prelude::*,
    sea_query::{Expr, SimpleExpr},
    Set,
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "direct_message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub icons: Option<String>,
}

impl Model {
    const MESSAGE: &'static str = "No DirectMessage with this expr";

    /// Return max id of `DirectMessage` Id
    ///
    /// # Errors
    /// Return an error if :
    /// * Database is unreachable
    pub async fn get_max_id(executor: &DatabaseConnection) -> Result<u32, String> {
        Ok(Model::get_model_by(Expr::col(Column::Id).max(), executor)
            .await?
            .id)
    }

    /// Get models with any condition
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist with this condition
    pub async fn get_model_by(
        simpl: SimpleExpr,
        executor: &DatabaseConnection,
    ) -> Result<Self, String> {
        match Entity::find().filter(simpl).one(executor).await {
            Ok(Some(model)) => Ok(model),
            _ => Err(Self::MESSAGE.to_string()),
        }
    }

    /// Get models with any condition
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist with this condition
    pub async fn get_models_by(
        simpl: SimpleExpr,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, String> {
        match Entity::find().filter(simpl).all(executor).await {
            Ok(model) => Ok(model),
            _ => Err(Self::MESSAGE.to_string()),
        }
    }

    /// Get models with any condition
    ///
    /// # Errors
    /// Return an error if:
    /// * Database is unreachable
    /// * Model doesn't exist with this condition
    pub async fn get_model_by_id(id: u32, executor: &DatabaseConnection) -> Result<Self, String> {
        match Entity::find().filter(Column::Id.eq(id)).one(executor).await {
            Ok(Some(model)) => Ok(model),
            _ => Err(Self::MESSAGE.to_string()),
        }
    }

    /// Return Model from a `DirectMessage`
    ///
    /// # Errors
    /// Return an error if :
    /// * Database is unreachable
    pub async fn from(dm: DirectMessage, executor: &DatabaseConnection) -> Result<Model, String> {
        Ok(Model {
            id: Model::get_max_id(executor).await? + 1,
            name: dm.name,
            icons: Some(dm.icons),
        })
    }
    pub fn to_directmessage(&self) -> DirectMessage {
        DirectMessage::new(
            Id::Id(self.id),
            self.name.clone(),
            self.icons.clone().unwrap_or_default(),
        )
    }
}
impl TryFrom<DirectMessage> for ActiveModel {
    type Error = String;

    fn try_from(value: DirectMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            name: Set(value.name.clone()),
            icons: Set(Some(value.icons)),
            ..Default::default()
        })
    }
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::direct_message_members::Entity")]
    DirectMessageMembers,
}

impl Related<super::direct_message_members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectMessageMembers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
