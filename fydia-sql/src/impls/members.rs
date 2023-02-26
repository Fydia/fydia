use super::insert;
use entity::members::*;
use fydia_struct::{
    response::FydiaResponse,
    server::{Members, ServerId},
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
    ) -> Result<Members, FydiaResponse>;
    async fn servers_of(
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, FydiaResponse>;
    async fn insert(
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlMembers for Members {
    async fn users_of(
        server: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, FydiaResponse> {
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
    ) -> Result<Vec<ServerId>, FydiaResponse> {
        let userid = userid.0.get_id_cloned_fydiaresponse()?;

        Ok(Model::get_models_by(Column::Userid.eq(userid), executor)
            .await?
            .iter()
            .map(|i| i.to_server())
            .collect())
    }

    async fn insert(
        server: &ServerId,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let acmodel = Model::new_activemodel(user, server.clone())?;

        insert(acmodel, executor).await.map(|_| ())
    }
}
