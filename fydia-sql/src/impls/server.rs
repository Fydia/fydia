use std::convert::TryFrom;

use fydia_struct::{
    channel::Channel,
    server::{Members, Server, ServerId},
    user::{User, UserId},
};
use sea_orm::{DatabaseConnection, Set};

use crate::entity::server::Model;

use super::{delete, insert, members::SqlMembers, update, user::UserFrom};

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
        Model::get_model_by_id(&id.id, executor)
            .await?
            .to_server(executor)
            .await
    }

    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut user = self
            .owner
            .get_user(executor)
            .await
            .ok_or_else(|| "Owner not found ?".to_string())?;

        let active_channel = crate::entity::server::ActiveModel::try_from(self.clone())?;

        insert(active_channel, executor).await?;

        self.join(&mut user, executor).await
    }

    async fn delete_server(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = crate::entity::server::ActiveModel::try_from(self.clone())?;

        delete(active_channel, executor).await
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

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = crate::entity::server::ActiveModel::try_from(self.clone())?;

        update(am, executor).await
    }

    async fn join(&mut self, user: &mut User, executor: &DatabaseConnection) -> Result<(), String> {
        Members::insert(&self.id, &user.id, executor).await?;

        user.insert_server(&self.id);

        self.members.members.push(user.id.clone());

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let active_channel = crate::entity::channels::ActiveModel::try_from(channel)?;

        insert(active_channel, executor).await?;

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
    async fn to_users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String>;
    async fn all_exists(&self, executor: &DatabaseConnection) -> bool;
}

#[async_trait::async_trait]
impl SqlMember for Members {
    async fn to_users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String> {
        let mut result = Vec::new();

        for i in &self.members {
            let user = i
                .get_user(executor)
                .await
                .ok_or_else(|| String::from("User not exists"))?;

            result.push(user);
        }

        Ok(result)
    }

    async fn all_exists(&self, executor: &DatabaseConnection) -> bool {
        for i in &self.members {
            if i.get_user(executor)
                .await
                .ok_or_else(|| String::from("User not exists"))
                .is_err()
            {
                return false;
            }
        }
        return true;
    }
}

#[async_trait::async_trait]
impl SqlMember for Vec<UserId> {
    async fn to_users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String> {
        let members = Members::new(self.clone());

        members.to_users(executor).await
    }

    async fn all_exists(&self, executor: &DatabaseConnection) -> bool {
        for i in self {
            if i.get_user(executor)
                .await
                .ok_or_else(|| String::from("User not exists"))
                .is_err()
            {
                return false;
            };
        }

        return true;
    }
}
