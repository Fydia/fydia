use fydia_struct::{
    channel::Channel,
    server::{Members, Server, ServerId},
    user::{User, UserId},
};
use fydia_utils::async_trait;
use sea_orm::{DatabaseConnection, Set};
use std::convert::TryFrom;

use entity::server::Model;

use super::{basic_model::BasicModel, delete, insert, members::SqlMembers, update, user::UserFrom};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, String>;
    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, String>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), String>;
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
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, String> {
        Members::users_of(&self.id, executor).await
    }

    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, String> {
        Model::get_model_by_id(&id.id, executor)
            .await?
            .to_struct(executor)
            .await
    }

    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let mut user = self
            .owner
            .to_user(executor)
            .await
            .ok_or_else(|| "Owner not found ?".to_string())?;

        let active_channel = entity::server::ActiveModel::try_from(self.clone())?;

        insert(active_channel, executor).await?;

        self.join(&mut user, executor).await
    }

    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = entity::server::ActiveModel::try_from(self.clone())?;

        delete(active_channel, executor).await
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let mut active_model: entity::server::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor).await?.into();

        active_model.name = Set(name.clone());

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = entity::server::ActiveModel::try_from(self.clone())?;

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
        let active_channel = entity::channels::ActiveModel::try_from(channel)?;

        insert(active_channel, executor).await?;

        self.channel.0.push(channel.clone());

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, String>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, String> {
        Server::by_id(&ServerId::new(self.id.clone()), executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlMember {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String>;
    async fn is_valid(&self, executor: &DatabaseConnection) -> bool;
}

#[async_trait::async_trait]
impl SqlMember for Members {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String> {
        let mut result = Vec::new();

        for id in &self.members {
            let user = id
                .to_user(executor)
                .await
                .ok_or_else(|| String::from("User not exists"))?;

            result.push(user);
        }

        Ok(result)
    }

    async fn is_valid(&self, executor: &DatabaseConnection) -> bool {
        for id in &self.members {
            if id
                .to_user(executor)
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
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, String> {
        let members = Members::new(self.clone());

        members.users(executor).await
    }

    async fn is_valid(&self, executor: &DatabaseConnection) -> bool {
        for id in self {
            if id
                .to_user(executor)
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
