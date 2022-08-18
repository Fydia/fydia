//! `SeoORM` Entity. Generated by sea-orm-codegen 0.6.0

use fydia_struct::{directmessage::DirectMessage, user::UserId, utils::Id};
use sea_orm::{entity::prelude::*, sea_query::SimpleExpr, Set};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "DirectMessageMembers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub directmessage: u32,
}

impl Model {
    const MESSAGE: &'static str = "No DirectMessage with this expr";

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

    pub fn to_userid(&self) -> UserId {
        UserId(Id::Id(self.user))
    }

    /// Return `DirectMessage`
    ///
    /// # Errors
    /// Return an error if :
    /// * Model with this id isn't exists
    pub async fn get_directmessage(
        &self,
        executor: &DatabaseConnection,
    ) -> Result<DirectMessage, String> {
        let direct_message =
            super::direct_message::Model::get_model_by_id(self.directmessage as u32, executor)
                .await?;

        Ok(direct_message.to_directmessage())
    }

    /// Return an activemodel from `userid`
    ///
    /// # Errors
    /// Return an error if :
    /// * Id is unset
    pub fn new_activemodel(
        userid: &UserId,
        directmessage: &DirectMessage,
    ) -> Result<ActiveModel, String> {
        Ok(ActiveModel {
            directmessage: Set(directmessage.id.get_id_cloned()?),
            user: Set(userid.0.get_id_cloned()?),
        })
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::User",
        to = "super::user::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::direct_message::Entity",
        from = "Column::Directmessage",
        to = "super::direct_message::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    DirectMessage,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::direct_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectMessage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
