use fydia_struct::{
    server::{Members, ServerId},
    user::UserId,
};
use sea_orm::{ColumnTrait, DatabaseConnection};

use crate::entity::members::*;

use super::{get_all, insert};

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
        let model: Vec<UserId> = get_all(
            Entity,
            vec![Column::Serverid.contains(&serverid.id)],
            executor,
        )
        .await?
        .iter()
        .map(|i| i.to_userid())
        .collect();

        Ok(Members::new_with(model.len() as i32, model))
    }

    async fn get_servers_by_usersid(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, String> {
        Ok(get_all(Entity, vec![Column::Userid.eq(userid.0)], executor)
            .await?
            .iter()
            .map(|i| i.to_server())
            .collect())
    }

    async fn insert(
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let acmodel = Model::new_activemodel(userid.clone(), serverid.clone());

        insert(acmodel, executor).await
    }
}
