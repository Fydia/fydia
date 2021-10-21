use crate::entity;
use crate::sqlpool::parse_array;
use fydia_struct::permission::Permission;
use fydia_struct::{roles::Role, server::ServerId};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn get_roles_by_server_id(
        shortid: String,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, String>;
    async fn get_role_by_id(role_id: i32, executor: &DatabaseConnection) -> Result<Role, String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_color(
        &mut self,
        color: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn delete_role(&self, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn get_roles_by_server_id(
        shortid: String,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, String> {
        let mut result = Vec::new();
        let query = crate::entity::roles::Entity::find()
            .filter(entity::roles::Column::Serverid.eq(shortid))
            .all(executor)
            .await;
        if let Ok(query) = query {
            for i in query {
                result.push(Self {
                    id: i.id,
                    server_id: ServerId::new(i.serverid),
                    name: i.name,
                    color: i.color,
                    channel_access: parse_array(
                        i.channel_access.unwrap_or_else(|| String::from("[]")),
                    ),
                    permission: Permission::from_string(i.permission.unwrap_or_default()),
                });
            }
        }

        Ok(result)
    }

    async fn get_role_by_id(role_id: i32, executor: &DatabaseConnection) -> Result<Self, String> {
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(role_id))
            .one(executor)
            .await;
        match query {
            Ok(Some(model)) => Ok(Role {
                id: model.id,
                name: model.name,
                color: model.color,
                server_id: ServerId::new(model.serverid),
                channel_access: parse_array(
                    model.channel_access.unwrap_or_else(|| String::from("[]")),
                ),
                permission: Permission::from_string(model.permission.unwrap_or_default()),
            }),
            Err(e) => Err(e.to_string()),
            _ => Err(String::from("No Role with this id")),
        }
    }

    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let active_model = entity::roles::ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        };

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(self.id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.name = name;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn update_color(
        &mut self,
        color: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let active_model = entity::roles::ActiveModel {
            color: Set(color.clone()),
            ..Default::default()
        };

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(self.id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.color = color;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    async fn delete_role(&self, executor: &DatabaseConnection) -> Result<(), String> {
        match entity::roles::Entity::find_by_id(self.id)
            .one(executor)
            .await
        {
            Ok(Some(model)) => {
                let active_model: entity::roles::ActiveModel = model.into();
                match entity::roles::Entity::delete(active_model)
                    .exec(executor)
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
            _ => Err(String::from("This role not exists")),
        }
    }
}
