use entity::members::*;
use fydia_struct::{
    response::FydiaResponse,
    server::{Members, ServerId},
    user::UserId,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection};

use super::insert;

#[async_trait::async_trait]
pub trait SqlMembers {
    async fn users_of<'a>(
        server: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, FydiaResponse<'a>>;
    async fn servers_of<'a>(
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, FydiaResponse<'a>>;
    async fn insert<'a>(
        serverid: &ServerId,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>>;
}

#[async_trait::async_trait]
impl SqlMembers for Members {
    async fn users_of<'a>(
        server: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Members, FydiaResponse<'a>> {
        let model: Vec<UserId> =
            Model::get_models_by(Column::Serverid.contains(&server.id), executor)
                .await?
                .iter()
                .map(|i| i.to_userid())
                .collect();

        Ok(Members::new(model))
    }

    async fn servers_of<'a>(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<ServerId>, FydiaResponse<'a>> {
        let userid = userid.0.get_id_cloned_fydiaresponse()?;

        Ok(Model::get_models_by(Column::Userid.eq(userid), executor)
            .await?
            .iter()
            .map(|i| i.to_server())
            .collect())
    }

    async fn insert<'a>(
        server: &ServerId,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>> {
        let acmodel = Model::new_activemodel(user, server.clone())?;

        insert(acmodel, executor).await.map(|_| ())
    }
}
