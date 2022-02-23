use fydia_struct::{
    channel::{Channel, ParentId},
    roles::Role,
    server::{Members, Server, ServerId},
    user::{User, UserId},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::server;

use super::{
    channel::SqlChannel,
    role::SqlRoles,
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
        match crate::entity::server::Entity::find()
            .filter(crate::entity::server::Column::Id.eq(self.id.id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => match serde_json::from_str::<Members>(&e.members) {
                Ok(value) => Ok(value),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
            _ => Err("".to_string()),
        }
    }

    async fn get_server_by_id(
        id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Server, String> {
        match crate::entity::server::Entity::find()
            .filter(server::Column::Id.eq(id.id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(model)) => {
                let members = serde_json::from_str::<Members>(model.members.as_str())
                    .map_err(|f| f.to_string())?;

                let roles = Role::get_roles_by_server_id(model.id.clone(), executor).await?;

                let channel = Channel::get_channels_by_server_id(id, executor).await?;

                Ok(Server {
                    id: ServerId::new(model.id),
                    name: model.name,
                    owner: UserId::new(model.owner),
                    icon: model.icon.unwrap_or_else(|| "Error".to_string()),
                    members,
                    channel,
                    roles,
                    ..Default::default()
                })
            }
            Err(e) => Err(e.to_string()),
            _ => Err("Cannot get server".to_string()),
        }
    }
    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let members_json = serde_json::to_string(&Members::new()).map_err(|f| f.to_string())?;

        let active_channel = crate::entity::server::ActiveModel {
            id: Set(self.id.id.clone()),
            name: Set(self.name.clone()),
            members: Set(members_json),
            owner: Set(self.owner.id),
            icon: Set(Some(self.icon.clone())),
        };
        match crate::entity::server::Entity::insert(active_channel)
            .exec(executor)
            .await
        {
            Ok(_) => {
                let mut user = self
                    .owner
                    .get_user(executor)
                    .await
                    .ok_or_else(|| "Owner is existing ?".to_string())?;

                self.join(&mut user, executor).await?;

                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete_server(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_channel = crate::entity::server::ActiveModel {
            id: Set(self.id.id.clone()),
            ..Default::default()
        };
        match crate::entity::server::Entity::delete(active_channel)
            .exec(executor)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        match crate::entity::server::Entity::find()
            .filter(server::Column::Id.contains(self.id.id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => {
                let mut active_model: crate::entity::server::ActiveModel = e.into();
                active_model.name = Set(name.clone());
                match crate::entity::server::Entity::update(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
            _ => Err("Can't update name".to_string()),
        }?;

        self.name = name;

        Ok(())
    }

    async fn update(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let am = crate::entity::server::ActiveModel {
            id: Set(self.id.id.clone()),
            name: Set(self.name.clone()),
            owner: Set(self.owner.id),
            icon: Set(Some(self.icon.clone())),
            members: Set(self.members.to_string()?),
        };
        match crate::entity::server::Entity::update(am)
            .exec(executor)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn join(&mut self, user: &mut User, executor: &DatabaseConnection) -> Result<(), String> {
        let server = match crate::entity::server::Entity::find()
            .filter(crate::entity::server::Column::Id.eq(self.id.id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => Ok(e),
            Err(e) => Err(e.to_string()),
            _ => Err("Cannot get server".to_string()),
        }?;

        let mut members = self.get_user(executor).await?;
        let mut to_push = user.to_userinfo();
        to_push.servers.0.push(self.id.clone());
        members.push(to_push);

        let json = members.to_string()?;

        let mut active_model: crate::entity::server::ActiveModel = server.into();

        active_model.members = Set(json);

        crate::entity::server::Entity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        user.insert_server(&self.id, executor).await?;

        self.members = members;

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: &Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let parent_id = match &channel.parent_id {
            ParentId::DirectMessage(_) => return Err(String::from("Bad type of Channel")),
            ParentId::ServerId(_) => ParentId::ServerId(self.id.clone()).to_string()?,
        };

        let active_channel = crate::entity::channels::ActiveModel {
            id: Set(channel.id.id.clone()),
            parent_id: Set(parent_id),
            name: Set(channel.name.clone()),
            description: Set(Some(channel.description.clone())),
            channel_type: Set(Some(channel.channel_type.to_string())),
        };

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
