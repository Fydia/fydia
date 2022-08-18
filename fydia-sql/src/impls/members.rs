use entity::members::*;
use fydia_struct::{
    server::{Members, ServerId},
    user::UserId,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection};

use super::insert;

#[async_trait::async_trait]
pub trait SqlMembers {
    async fn users_of(server: &ServerId, executor: &DatabaseConnection) -> Result<Members, String>;
    async fn servers_of(
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, String>;
    async fn insert(
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlMembers for Members {
    async fn users_of(server: &ServerId, executor: &DatabaseConnection) -> Result<Members, String> {
        let model: Vec<UserId> =
            Model::get_models_by(Column::Serverid.contains(&server.id), executor)
                .await?
                .iter()
                .map(|i| i.to_userid())
                .collect();

        Ok(Members::new(model))
    }

    async fn servers_of(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, String> {
        Ok(
            Model::get_models_by(Column::Userid.eq(userid.0.get_id_cloned()?), executor)
                .await?
                .iter()
                .map(|i| i.to_server())
                .collect(),
        )
    }

    async fn insert(
        server: &ServerId,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let acmodel = Model::new_activemodel(user, server.clone())?;

        insert(acmodel, executor).await
    }
}
