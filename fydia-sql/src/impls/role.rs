use crate::entity;
use fydia_struct::roles::Role;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Set};

use super::{delete, get_all, get_one, update};

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn get_roles_by_server_id(
        shortid: String,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, String>;
    async fn get_role_by_id(role_id: i32, executor: &DatabaseConnection) -> Result<Role, String>;
    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_color<T: Into<String> + Send>(
        &mut self,
        color: T,
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
        Ok(get_all(
            entity::roles::Entity,
            vec![entity::roles::Column::Serverid.eq(shortid)],
            executor,
        )
        .await?
        .iter()
        .map(|f| f.to_role())
        .collect::<Vec<Self>>())
    }

    async fn get_role_by_id(role_id: i32, executor: &DatabaseConnection) -> Result<Self, String> {
        Ok(get_one(
            entity::roles::Entity,
            vec![entity::roles::Column::Id.eq(role_id)],
            executor,
        )
        .await?
        .to_role())
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let active_model = entity::roles::ActiveModel {
            id: Set(self.id),
            name: Set(name.clone()),
            ..Default::default()
        };

        update(active_model, executor).await?;

        self.name = name;

        Ok(())
    }

    async fn update_color<T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let color = color.into();
        let active_model = entity::roles::ActiveModel {
            id: Set(self.id),
            color: Set(color.clone()),
            ..Default::default()
        };

        update(active_model, executor).await?;

        self.color = color;

        Ok(())
    }

    async fn delete_role(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = entity::roles::Entity::find_by_id(self.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or("Can't the role")?;

        let active_model: entity::roles::ActiveModel = model.into();

        delete(active_model, executor).await
    }
}
