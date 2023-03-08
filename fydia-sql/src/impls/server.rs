use entity::server::Model;
use fydia_struct::{
    channel::Channel,
    response::{FydiaResponse, MapError},
    server::{Channels, Members, Server, ServerId},
    user::{User, UserId},
};
use fydia_utils::async_trait;
use sea_orm::{DatabaseConnection, Set};
use shared::sea_orm;
use std::convert::TryFrom;

use super::{
    basic_model::BasicModel, channel::SqlChannel, delete, insert, members::SqlMembers, update,
    user::UserFrom,
};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, FydiaResponse>;
    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, FydiaResponse>;
    async fn channels(&self, executor: &DatabaseConnection) -> Result<Channels, FydiaResponse>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn update(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn join(
        &mut self,
        user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, FydiaResponse> {
        Members::users_of(&self.id, executor).await
    }

    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, FydiaResponse> {
        Model::get_model_by_id(&id.id, executor)
            .await?
            .to_struct(executor)
            .await
    }

    async fn channels(&self, executor: &DatabaseConnection) -> Result<Channels, FydiaResponse> {
        Channel::by_serverid(&self.id, executor).await
    }

    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let mut user = self.owner.to_user(executor).await?;

        let active_channel =
            entity::server::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        insert(active_channel, executor).await?;

        self.join(&mut user, executor).await
    }

    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let active_channel =
            entity::server::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        delete(active_channel, executor).await
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let name = name.into();
        let mut active_model: entity::server::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor).await?.into();

        active_model.name = Set(name.clone());

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let am = entity::server::ActiveModel::try_from(self.clone()).error_to_fydiaresponse()?;

        update(am, executor).await
    }

    async fn join(
        &mut self,
        user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        Members::insert(&self.id, &user.id, executor).await?;

        user.insert_server(&self.id);

        self.members.members.push(user.id.clone());

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let active_channel =
            entity::channels::ActiveModel::try_from(channel).error_to_fydiaresponse()?;

        insert(active_channel, executor).await?;

        self.channel.0.push(channel.clone());

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, FydiaResponse> {
        Server::by_id(self, executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlMember {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlMember for Members {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, FydiaResponse> {
        let mut result = Vec::new();

        for id in &self.members {
            let user = id.to_user(executor).await?;

            result.push(user);
        }

        Ok(result)
    }
}

#[async_trait::async_trait]
impl SqlMember for Vec<UserId> {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, FydiaResponse> {
        let members = Members::new(self.clone());

        members.users(executor).await
    }
}
