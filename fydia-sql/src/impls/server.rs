use fydia_struct::{
    channel::{Channel, ParentId},
    roles::Role,
    server::{Members, Server, ServerId},
    user::{User, UserId},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::server;

use super::{channel::SqlChannel, role::SqlRoles, user::{SqlUser, UserIdSql}};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn get_user(&self, executor: &DatabaseConnection) -> Result<Members, String>;
    async fn get_server_by_id(
        id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Server, String>;
    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn delete_server(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn join(
        &mut self,
        mut user: &mut User,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn insert_channel(
        &mut self,
        channel: Channel,
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
                Ok(value) => return Ok(value),
                Err(e) => {
                    {
                        error!("Error");
                        return Err(e.to_string());
                    };
                }
            },
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => return Err("".to_string()),
        }
    }

    async fn get_server_by_id(
        id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Server, String> {
        match crate::entity::server::Entity::find()
            .filter(server::Column::Shortid.eq(id.short_id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(model)) => {
                let members = match serde_json::from_str::<Members>(model.members.as_str()) {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                };

                let roles =
                    match Role::get_roles_by_server_id(model.shortid.clone(), executor).await {
                        Ok(e) => e,
                        Err(e) => return Err(e),
                    };

                let channel =
                    match Channel::get_channels_by_server_id(id, executor).await
                    {
                        Ok(e) => e,
                        Err(e) => return Err(e),
                    };

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
            Err(e) => {
                return Err(e.to_string());
            }
            _ => {
                return Err("Cannot get server".to_string());
            }
        }
    }
    async fn insert_server(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let members_json = match serde_json::to_string(&Members::new()) {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        let active_channel = crate::entity::server::ActiveModel {
            id: Set(self.id.id.clone()),
            shortid: Set(self.id.short_id.clone()),
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
                let mut user = self.owner.get_user(executor).await.ok_or("Owner is existing ?".to_string())?;
                self.join(&mut user, executor).await?;

                Ok(())
            },
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

    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match crate::entity::server::Entity::find()
            .filter(server::Column::Shortid.contains(self.id.short_id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => {
                let mut active_model: crate::entity::server::ActiveModel = e.into();
                active_model.name = Set(name);
                match crate::entity::server::Entity::update(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                };
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => {}
        };

        self.name = name;

        Ok(())
    }

    async fn join(&mut self, user: &mut User, executor: &DatabaseConnection) -> Result<(), String> {
        let server = match crate::entity::server::Entity::find()
            .filter(crate::entity::server::Column::Shortid.eq(self.id.short_id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => e,
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => {
                error!("Error");
                return Err("Cannot get server".to_string());
            }
        };

        let mut members = match self.get_user(executor).await {
            Ok(vec_users) => vec_users,
            Err(e) => {
                error!("Error");
                return Err(e);
            }
        };

        members.push(user.clone());

        let json = match serde_json::to_string(&members) {
            Ok(json) => json,
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        };

        println!("{}", json);

        let mut active_model: crate::entity::server::ActiveModel = server.into();

        active_model.members = Set(json);

        match crate::entity::server::Entity::update(active_model)
            .exec(executor)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        }

        if let Err(error) = user.insert_server(self.id.clone(), executor).await {
            return Err(error);
        }

        self.members = members;

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: Channel,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let parent_id = match channel.parent_id.clone() {
            ParentId::DirectMessage(_) => return Err(String::from("Bad type of Channel")),
            ParentId::ServerId(_) => {
                ParentId::ServerId(self.id.clone()).to_string()?
            }
        };
        let active_channel = crate::entity::channels::ActiveModel {
            id: Set(channel.id.id.clone()),
            parent_id: Set(parent_id),
            name: Set(channel.name.clone()),
            description: Set(Some(channel.description.clone())),
            channel_type: Set(Some(channel.channel_type.to_string())),
        };
        match crate::entity::channels::Entity::insert(active_channel)
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.channel.0.push(channel);
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

#[async_trait::async_trait]
pub trait SqlServerId {
    async fn get_server(&self, executor: &DatabaseConnection) -> Result<Server, String>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get_server(&self, executor: &DatabaseConnection) -> Result<Server, String> {
        Server::get_server_by_id(ServerId::new(self.id.clone()), executor).await
    }
}
