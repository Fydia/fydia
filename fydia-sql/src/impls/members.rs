use super::insert;
use entity::members::{Column, Model};
use fydia_struct::{
    server::{Members, MembersError, ServerId},
    user::UserId,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection};
use shared::sea_orm;

#[async_trait::async_trait]
pub trait SqlMembers {
    async fn users_of(
        server: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, MembersError>;
    async fn servers_of(
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, MembersError>;
    async fn insert(
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), MembersError>;
}

#[async_trait::async_trait]
impl SqlMembers for Members {
    async fn users_of(
        server: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, MembersError> {
        let model: Vec<UserId> =
            Model::get_models_by(Column::Serverid.contains(&server.id), executor)
                .await?
                .iter()
                .map(Model::to_userid)
                .collect();

        Ok(Members::new(model))
    }

    async fn servers_of(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, MembersError> {
        let userid = userid.0.get_id_cloned()?;

        Ok(Model::get_models_by(Column::Userid.eq(userid), executor)
            .await?
            .iter()
            .map(Model::to_server)
            .collect())
    }

    async fn insert(
        server: &ServerId,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), MembersError> {
        let acmodel = Model::new_activemodel(user, server.clone())?;

        insert(acmodel, executor).await.map(|_| {})?;

        Ok(())
    }
}
