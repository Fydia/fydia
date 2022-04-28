//! `SeaORM` Entity

use fydia_struct::{server::ServerId, user::UserId};
use sea_orm::{entity::prelude::*, sea_query::SimpleExpr, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "Members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub serverid: String,
    pub userid: i32,
}

impl Model {
    const MESSAGE: &'static str = "No Member with this expr";

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
        UserId(self.userid)
    }

    pub fn to_server(&self) -> ServerId {
        ServerId::new(self.serverid.clone())
    }

    pub fn new_activemodel(userid: &UserId, serverid: ServerId) -> ActiveModel {
        ActiveModel {
            serverid: Set(serverid.id),
            userid: Set(userid.0),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Userid",
        to = "super::user::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::server::Entity",
        from = "Column::Serverid",
        to = "super::server::Column::Id",
        on_update = "Restrict",
        on_delete = "Restrict"
    )]
    Server,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Server.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
