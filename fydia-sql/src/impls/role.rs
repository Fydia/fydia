use entity::roles::assignation;
use fydia_struct::{roles::Role, server::ServerId, user::UserId, utils::Id};

use super::{delete, insert};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn by_server_id(
        shortid: &String,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, String>;
    async fn by_id(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Role, String>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
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
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn add_user(&self, userid: &UserId, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn by_server_id(
        shortid: &String,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, String> {
        let mut result = Vec::new();
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Serverid.eq(shortid.as_str()))
            .all(executor)
            .await;
        if let Ok(query) = query {
            for i in query {
                result.push(i.to_role());
            }
        }

        Ok(result)
    }

    async fn by_id(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Self, String> {
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(role_id))
            .filter(entity::roles::Column::Serverid.eq(shortid.id.as_str()))
            .one(executor)
            .await;
        match query {
            Ok(Some(model)) => Ok(model.to_role()),
            Err(e) => Err(e.to_string()),
            _ => Err(String::from("No Role with this id")),
        }
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let active_model = entity::roles::ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        };

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(self.id.get_id_cloned()?))
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

    async fn update_color<T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let color = color.into();
        let active_model = entity::roles::ActiveModel {
            color: Set(color.clone()),
            ..Default::default()
        };

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(self.id.get_id_cloned()?))
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
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = entity::roles::ActiveModel::from(self.clone());

        let result = insert(model, executor).await?;

        self.id = Id::Id(result.last_insert_id);

        Ok(())
    }
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = entity::roles::Entity::find_by_id(self.id.get_id_cloned()?)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or("Can't the role")?;

        let active_model: entity::roles::ActiveModel = model.into();

        delete(active_model, executor).await
    }

    async fn add_user(&self, userid: &UserId, executor: &DatabaseConnection) -> Result<(), String> {
        let am = assignation::ActiveModel {
            role_id: Set(self.id.get_id_cloned()?),
            user_id: Set(userid.0.get_id_cloned()?),
            server_id: Set(self.server_id.id.clone()),
        };

        insert(am, executor).await.map(|_| ())
    }
}
