use std::convert::TryFrom;

use crate::{entity::direct_message as dm, entity::direct_message_members as dm_members};
use fydia_struct::{directmessage::DirectMessage, server::Members, user::UserId, utils::Id};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait};

use super::{delete, insert, user::UserFrom};
#[async_trait::async_trait]
pub trait DirectMessageMembers {
    async fn get_members(&self, executor: &DatabaseConnection) -> Result<Members, String>;
    async fn get_servers_of(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, String>;
    async fn insert_user(&self, user: &UserId, executor: &DatabaseConnection)
        -> Result<(), String>;
    async fn delete_user(&self, user: &UserId, executor: &DatabaseConnection)
        -> Result<(), String>;
}

#[async_trait::async_trait]
impl DirectMessageMembers for DirectMessage {
    async fn get_members(&self, executor: &DatabaseConnection) -> Result<Members, String> {
        let userids = dm_members::Model::get_models_by(
            dm_members::Column::Directmessage.eq(self.id.get_id_cloned()?),
            executor,
        )
        .await?
        .iter()
        .map(|f| f.to_userid())
        .collect::<Vec<UserId>>();

        Ok(Members::new(userids))
    }
    async fn get_servers_of(
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<Vec<DirectMessage>, String> {
        let mut result = Vec::new();

        for i in dm_members::Model::get_models_by(
            dm_members::Column::User.eq(userid.0.get_id_cloned()?),
            executor,
        )
        .await?
        {
            result.push(i.get_directmessage(executor).await?);
        }

        Ok(result)
    }
    async fn insert_user(
        &self,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        if user.get_user(executor).await.is_none() {
            return Err("User not exists".to_string());
        };

        insert(dm_members::Model::new_activemodel(user, self)?, executor).await
    }

    async fn delete_user(
        &self,
        user: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let members = self.get_members(executor).await?;

        for i in members.members {
            if &i == user {
                delete(dm_members::Model::new_activemodel(user, self)?, executor).await?;
                return Ok(());
            }
        }

        Err("User is not a member of this direct_message".to_string())
    }
}

#[async_trait::async_trait]
pub trait SqlDirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn get(dm_id: Id<u32>, executor: &DatabaseConnection) -> Result<DirectMessage, String>;
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlDirectMessage for DirectMessage {
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = dm::ActiveModel::try_from(self.clone())?;
        let result = dm::Entity::insert(am)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.id.set(result.last_insert_id);

        Ok(())
    }
    async fn get(dm_id: Id<u32>, executor: &DatabaseConnection) -> Result<DirectMessage, String> {
        Ok(
            dm::Model::get_model_by(dm::Column::Id.eq(dm_id.get_id()?), executor)
                .await?
                .to_directmessage(),
        )
    }
    async fn delete(self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = dm::ActiveModel::try_from(self.clone())?;
        delete(am, executor).await
    }
}