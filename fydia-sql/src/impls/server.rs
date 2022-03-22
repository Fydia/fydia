use std::convert::TryFrom;

use fydia_struct::{
    channel::{Channel, ParentId},
    server::{Members, Server, ServerId},
    user::{User, UserId, UserInfo},
};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use crate::entity::server::Model;

use super::{
    members::SqlMembers,
    user::{SqlUser, UserFrom},
};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn get_user(&self, executor: &DatabaseConnection) -> Result<Members, String>;
    async fn get_server_by_id(
        id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Server, String>;
    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn delete_server(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn join(
        &mut self,
        mut user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn get_user(&self, executor: &DatabaseConnection) -> Result<Members, String> {
        Members::get_users_by_serverid(&self.id, executor).await
    }

    async fn get_server_by_id(
        id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Server, String> {
        let model = Model::get_model_by_id(&id.id, executor).await?;

        model.to_server(executor).await
    }

    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut user = self
            .owner
            .get_user(executor)
            .await
            .ok_or_else(|| "Owner not found ?".to_string())?;

        let active_channel = crate::entity::server::ActiveModel::try_from(self.clone())?;

        crate::entity::server::Entity::insert(active_channel)
            .exec(executor)
            .await
            .map_err(|error| error.to_string())?;

        self.join(&mut user, executor).await
    }

    async fn delete_server(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = crate::entity::server::ActiveModel::try_from(self.clone())?;

        if let Err(error) = crate::entity::server::Entity::delete(active_channel)
            .exec(executor)
            .await
        {
            return Err(error.to_string());
        }

        Ok(())
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let mut active_model: crate::entity::server::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor).await?.into();
        active_model.name = Set(name.clone());

        crate::entity::server::Entity::update(active_model)
            .exec(executor)
            .await
            .map_err(|e| e.to_string())?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = crate::entity::server::ActiveModel::try_from(self.clone())?;
        match crate::entity::server::Entity::update(am)
            .exec(executor)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn join(&mut self, user: &mut User, executor: &DatabaseConnection) -> Result<(), String> {
        Members::insert(&self.id, &user.id, executor).await?;

        user.insert_server(&self.id)?;

        self.members.push(user.id.clone());

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        if let ParentId::DirectMessage(_) = channel.parent_id {
            return Err(String::from("Bad type of Channel"));
        }

        let active_channel = crate::entity::channels::ActiveModel::try_from(channel)?;

        crate::entity::channels::Entity::insert(active_channel)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.channel.0.push(channel.clone());

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get_server(&self, executor: &DatabaseConnection) -> Result<Server, String>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get_server(&self, executor: &DatabaseConnection) -> Result<Server, String> {
        Server::get_server_by_id(&ServerId::new(self.id.clone()), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlMember {
    async fn to_userinfo(&self, executor: &DatabaseConnection) -> Result<Vec<UserInfo>, String>;
}

#[async_trait::async_trait]
impl SqlMember for Members {
    async fn to_userinfo(&self, executor: &DatabaseConnection) -> Result<Vec<UserInfo>, String> {
        let mut result = Vec::new();
        for i in &self.members {
            let user = i
                .get_user(executor)
                .await
                .ok_or_else(|| String::from("User not exists"))?
                .to_userinfo();
            result.push(user);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl SqlMember for Vec<UserId> {
    async fn to_userinfo(&self, executor: &DatabaseConnection) -> Result<Vec<UserInfo>, String> {
        let members = Members::new_with(self.len() as i32, self.clone());

        members.to_userinfo(executor).await
    }
}
