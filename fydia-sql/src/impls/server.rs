use std::sync::Arc;

use fydia_struct::{
    channel::Channel,
    instance::Instance,
    server::{Members, Server, ServerId},
    user::User,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::entity::server::{self};

use super::{channel::SqlChannel, user::SqlUser};

#[async_trait::async_trait]
pub trait SqlServer {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Result<Vec<User>, String>;
    async fn get_server_by_id(
        id: ServerId,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Server, String>;
    async fn insert_server(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String>;
    async fn delete_server(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
    async fn join(
        &mut self,
        mut user: &mut User,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
    async fn insert_channel(
        &mut self,
        channel: Channel,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlServer for Server {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Result<Vec<User>, String> {
        match crate::entity::server::Entity::find()
            .filter(crate::entity::server::Column::Id.eq(self.id.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => match serde_json::from_str::<serde_json::value::Value>(&e.members) {
                Ok(value) => {
                    if let Some(e) = value.get("members") {
                        if let Some(e) = e.as_array() {
                            let mut result = Vec::new();
                            for i in e {
                                match (i.get("id"), i.get("name")) {
                                    (Some(id), Some(name)) => match (id.as_str(), name.as_str()) {
                                        (Some(_), Some(name)) => result.push(User::new(
                                            name,
                                            "",
                                            "",
                                            Instance::default(),
                                        )),
                                        _ => {
                                            return Err("Json error".to_string());
                                        }
                                    },
                                    _ => {
                                        return Err("Json error".to_string());
                                    }
                                }
                            }

                            return Ok(result);
                        } else {
                            return Err("Json error".to_string());
                        }
                    } else {
                        return Err("Json error".to_string());
                    }
                }
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
        executor: &Arc<DatabaseConnection>,
    ) -> Result<Server, String> {
        match crate::entity::server::Entity::find()
            .filter(server::Column::Shortid.eq(id.short_id))
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

                /*let roles = match Role::get_roles_by_server_id(model.shortid, executor).await {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };*/

                let channel =
                    match Channel::get_channels_by_server_id(model.shortid.clone(), executor).await
                    {
                        Ok(e) => e,
                        Err(e) => return Err(e),
                    };

                Ok(Server {
                    id: model.id,
                    shortid: model.shortid,
                    name: model.name,
                    owner: model.owner,
                    icon: model.icon.unwrap_or_else(|| "Error".to_string()),
                    members,
                    channel,
                    ..Default::default()
                })
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => {
                error!("Error");
                return Err("Cannot get server".to_string());
            }
        }
    }
    async fn insert_server(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String> {
        let members_json = match serde_json::to_string(&Members::new()) {
            Ok(e) => e,
            Err(e) => return Err(e.to_string()),
        };
        let active_channel = crate::entity::server::ActiveModel {
            id: Set(self.id.clone()),
            name: Set(self.name.clone()),
            members: Set(members_json),
            shortid: Set(self.shortid.clone()),
            owner: Set(self.owner),
            icon: Set(Some(self.icon.clone())),
        };
        match crate::entity::server::Entity::insert(active_channel)
            .exec(executor)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete_server(&self, executor: &Arc<DatabaseConnection>) -> Result<(), String> {
        let active_channel = crate::entity::server::ActiveModel {
            id: Set(self.id.clone()),
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
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        match crate::entity::server::Entity::find()
            .filter(server::Column::Shortid.contains(self.shortid.as_str()))
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

    async fn join(
        &mut self,
        user: &mut User,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        let server = match crate::entity::server::Entity::find()
            .filter(crate::entity::server::Column::Shortid.eq(self.shortid.as_str()))
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

        let mut vecuser = match self.get_user(executor).await {
            Ok(vec_users) => vec_users,
            Err(e) => {
                error!("Error");
                return Err(e);
            }
        };

        vecuser.push(user.clone());

        let value = Members::new_with(vecuser.len() as i32, vecuser);
        let json = match serde_json::to_string(&value) {
            Ok(json) => json,
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        };

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

        if let Err(error) = user
            .insert_server(ServerId::new(self.id.clone()), executor)
            .await
        {
            return Err(error);
        }

        self.members = value;

        Ok(())
    }

    async fn insert_channel(
        &mut self,
        channel: Channel,
        executor: &Arc<DatabaseConnection>,
    ) -> Result<(), String> {
        let active_channel = crate::entity::channels::ActiveModel {
            id: Set(channel.id.clone()),
            serverid: Set(channel.server_id.short_id.clone()),
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
    async fn get_server(&self, executor: &Arc<DatabaseConnection>) -> Result<Server, String>;
}

#[async_trait::async_trait]
impl SqlServerId for ServerId {
    async fn get_server(&self, executor: &Arc<DatabaseConnection>) -> Result<Server, String> {
        Server::get_server_by_id(ServerId::new(self.id.clone()), executor).await
    }
}
