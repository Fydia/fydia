use std::convert::TryFrom;

use super::{delete, insert, user::UserFrom};
use fydia_struct::{
    directmessage::DirectMessage,
    response::{FydiaResponse, IntoFydia, MapError},
    server::Members,
    user::UserId,
    utils::Id,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use shared::sea_orm;
use {entity::direct_message as dm, entity::direct_message_members as dm_members};
#[async_trait::async_trait]
pub trait DirectMessageMembers {
    async fn members(&self, executor: &DatabaseConnection) -> Result<Members, FydiaResponse>;
    async fn of_user(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, FydiaResponse>;
    async fn add(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn remove(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl DirectMessageMembers for DirectMessage {
    async fn members(&self, executor: &DatabaseConnection) -> Result<Members, FydiaResponse> {
        let directmessageid = self.id.get_id_cloned_fydiaresponse()?;

        let userids = dm_members::Model::get_models_by(
            dm_members::Column::Directmessage.eq(directmessageid),
            executor,
        )
        .await?
        .iter()
        .map(|f| f.to_userid())
        .collect::<Vec<UserId>>();

        Ok(Members::new(userids))
    }
    async fn of_user(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, FydiaResponse> {
        let mut result = Vec::new();
        let userid = userid.0.get_id_cloned_fydiaresponse()?;

        for i in
            dm_members::Model::get_models_by(dm_members::Column::User.eq(userid), executor).await?
        {
            result.push(i.get_directmessage(executor).await?);
        }

        Ok(result)
    }
    async fn add(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        userid.to_user(executor).await?;

        insert(
            dm_members::Model::new_activemodel(userid, self).error_to_fydiaresponse()?,
            executor,
        )
        .await
        .map(|_| ())
    }

    async fn remove(
        &self,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let members = self.members(executor).await?;

        for i in members.members {
            if &i == user {
                delete(
                    dm_members::Model::new_activemodel(user, self).error_to_fydiaresponse()?,
                    executor,
                )
                .await?;
                return Ok(());
            }
        }

        Err("User is not a member of this direct_message".into_error())
    }
}

#[async_trait::async_trait]
pub trait SqlDirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn get(
        dm_id: Id<u32>,
        executor: &DatabaseConnection,
    ) -> Result<DirectMessage, FydiaResponse>;
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlDirectMessage for DirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let am = dm::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;
        let result = dm::Entity::insert(am)
            .exec(executor)
            .await
            .error_to_fydiaresponse()?;

        self.id.set(result.last_insert_id);

        Ok(())
    }
    async fn get(
        dm_id: Id<u32>,
        executor: &DatabaseConnection,
    ) -> Result<DirectMessage, FydiaResponse> {
        let id = dm_id.get_id().error_to_fydiaresponse()?;
        Ok(dm::Model::get_model_by(dm::Column::Id.eq(id), executor)
            .await?
            .to_directmessage())
    }
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let am = dm::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;
        delete(am, executor).await
    }
}
