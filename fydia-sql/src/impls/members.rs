use fydia_struct::{
    server::{Members, ServerId},
    user::UserId,
};
use sea_orm::{ColumnTrait, DatabaseConnection};

use entity::members::*;

use super::insert;

#[async_trait::async_trait]
pub trait SqlMembers {
    async fn get_users_by_serverid(
        serverid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, String>;
    async fn get_servers_by_usersid(
        serverid: &UserId,
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
    async fn get_users_by_serverid(
        serverid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, String> {
        let model: Vec<UserId> =
            Model::get_models_by(Column::Serverid.contains(&serverid.id), executor)
                .await?
                .iter()
                .map(|i| i.to_userid())
                .collect();

        Ok(Members::new(model))
    }

    async fn get_servers_by_usersid(
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
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let acmodel = Model::new_activemodel(userid, serverid.clone())?;

        insert(acmodel, executor).await
    }
}
