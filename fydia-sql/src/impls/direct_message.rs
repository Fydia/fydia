use std::convert::TryFrom;

use super::{delete, get_set_column, insert, user::UserFrom};
use fydia_struct::{
    directmessage::{DirectMessage, DirectMessageError},
    server::Members,
    sqlerror::{GenericError, GenericSqlError},
    user::UserId,
    utils::Id,
};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};
use shared::sea_orm;
use {entity::direct_message as dm, entity::direct_message_members as dm_members};
#[async_trait::async_trait]
pub trait DirectMessageMembers {
    async fn members(&self, executor: &DatabaseConnection) -> Result<Members, DirectMessageError>;
    async fn of_user(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, DirectMessageError>;
    async fn add(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), DirectMessageError>;
    async fn remove(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), DirectMessageError>;
}

#[async_trait::async_trait]
impl DirectMessageMembers for DirectMessage {
    async fn members(&self, executor: &DatabaseConnection) -> Result<Members, DirectMessageError> {
        let directmessageid = self.id.get_id_cloned()?;

        let userids = dm_members::Model::get_models_by(
            dm_members::Column::Directmessage.eq(directmessageid),
            executor,
        )
        .await?
        .iter()
        .map(dm_members::Model::to_userid)
        .collect::<Vec<UserId>>();

        Ok(Members::new(userids))
    }
    async fn of_user(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, DirectMessageError> {
        let mut result = Vec::new();
        let userid = userid.0.get_id_cloned()?;

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
    ) -> Result<(), DirectMessageError> {
        userid.to_user(executor).await?;

        insert(dm_members::Model::new_activemodel(userid, self)?, executor).await?;

        Ok(())
    }

    async fn remove(
        &self,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), DirectMessageError> {
        let members = self.members(executor).await?;

        for i in members.members {
            if &i == user {
                delete(dm_members::Model::new_activemodel(user, self)?, executor).await?;

                return Ok(());
            }
        }

        Err(DirectMessageError::UserNotInDm)
    }
}

#[async_trait::async_trait]
pub trait SqlDirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), DirectMessageError>;
    async fn get(
        dm_id: Id<u32>,
        executor: &DatabaseConnection,
    ) -> Result<DirectMessage, DirectMessageError>;
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), DirectMessageError>;
}

#[async_trait::async_trait]
impl SqlDirectMessage for DirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), DirectMessageError> {
        let am = dm::ActiveModel::try_from(self.clone())?;
        let set_column = get_set_column(&am);
        let result = dm::Entity::insert(am).exec(executor).await.map_err(|f| {
            GenericSqlError::CannotInsert(GenericError {
                set_column,
                error: f.to_string(),
            })
        })?;

        self.id.set(result.last_insert_id);

        Ok(())
    }
    async fn get(
        dm_id: Id<u32>,
        executor: &DatabaseConnection,
    ) -> Result<DirectMessage, DirectMessageError> {
        let id = dm_id.get_id()?;
        Ok(dm::Model::get_model_by_id(id, executor)
            .await?
            .to_directmessage())
    }
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), DirectMessageError> {
        let am = dm::ActiveModel::try_from(self.clone())?;

        delete(am, executor).await?;

        Ok(())
    }
}
