use entity::server::Model;
use fydia_struct::{
    channel::{Channel, ChannelError},
    server::{Channels, Members, MembersError, Server, ServerError, ServerId},
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
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, ServerError>;
    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, ServerError>;
    async fn channels(&self, executor: &DatabaseConnection) -> Result<Channels, ServerError>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), ServerError>;
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), ServerError>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), ServerError>;
    async fn update(&self, executor: &DatabaseConnection) -> Result<(), ServerError>;
    async fn join(
        &mut self,
        user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), ServerError>;
    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), ChannelError>;
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Members, ServerError> {
        let members = Members::users_of(&self.id, executor).await?;
        Ok(members)
    }

    async fn by_id(id: &ServerId, executor: &DatabaseConnection) -> Result<Server, ServerError> {
        let server = Model::get_model_by_id(&id.id, executor)
            .await?
            .to_struct(executor)
            .await?;

        Ok(server)
    }

    async fn channels(&self, executor: &DatabaseConnection) -> Result<Channels, ServerError> {
        let channels = Channel::by_serverid(&self.id, executor).await?;

        Ok(channels)
    }

    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), ServerError> {
        let mut user = self.owner.to_user(executor).await?;

        let active_channel = entity::server::ActiveModel::try_from(self.clone())?;

        insert(active_channel, executor).await?;

        self.join(&mut user, executor).await
    }

    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), ServerError> {
        let active_channel = entity::server::ActiveModel::try_from(self.clone())?;

        delete(active_channel, executor).await?;

        Ok(())
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), ServerError> {
        let name = name.into();
        let mut active_model: entity::server::ActiveModel =
            Model::get_model_by_id(&self.id.id, executor).await?.into();

        active_model.name = Set(name.clone());

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), ServerError> {
        let am = entity::server::ActiveModel::try_from(self.clone())?;

        update(am, executor).await?;

        Ok(())
    }

    async fn join(
        &mut self,
        user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), ServerError> {
        Members::insert(&self.id, &user.id, executor).await?;

        user.insert_server(&self.id);

        self.members.members.push(user.id.clone());

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), ChannelError> {
        let active_channel = entity::channels::ActiveModel::try_from(channel)?;

        insert(active_channel, executor).await?;

        self.channel.0.push(channel.clone());

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, ServerError>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get(&self, executor: &DatabaseConnection) -> Result<Server, ServerError> {
        Server::by_id(self, executor).await
    }
}

#[async_trait::async_trait]
pub trait SqlMember {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, MembersError>;
}

#[async_trait::async_trait]
impl SqlMember for Members {
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, MembersError> {
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
    async fn users(&self, executor: &DatabaseConnection) -> Result<Vec<User>, MembersError> {
        let members = Members::new(self.clone());

        members.users(executor).await
    }
}
