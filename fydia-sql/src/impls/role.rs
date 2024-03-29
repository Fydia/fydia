use entity::roles::assignation;
use fydia_struct::{
    roles::{Role, RoleError},
    server::ServerId,
    sqlerror::GenericSqlError,
    user::UserId,
    utils::Id,
};

use super::{delete, from_dberr_and_activemodel, get_set_column, insert};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use shared::sea_orm;
#[async_trait::async_trait]
pub trait SqlRoles {
    async fn by_server_id(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, RoleError>;
    async fn by_id(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Role, RoleError>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), RoleError>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError>;
    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError>;
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), RoleError>;
    async fn add_user(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn by_server_id(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, RoleError> {
        let mut result = Vec::new();
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Serverid.eq(shortid))
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
    ) -> Result<Self, RoleError> {
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(role_id))
            .filter(entity::roles::Column::Serverid.eq(shortid.id.as_str()))
            .one(executor)
            .await;

        match query {
            Ok(Some(model)) => Ok(model.to_role()),
            Err(e) => {
                error!("{}", e.to_string());
                Err(RoleError::NoRoleWithId)
            }
            _ => Err(RoleError::NoRoleWithId),
        }
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError> {
        let name = name.into();
        let active_model = entity::roles::ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        };

        let id = self.id.get_id_cloned()?;
        let set_column = get_set_column(&active_model);

        entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
            .map_err(|f| {
                GenericSqlError::CannotUpdate(from_dberr_and_activemodel(set_column, &f))
            })?;

        self.name = name;

        Ok(())
    }

    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError> {
        let color = color.into();
        let active_model = entity::roles::ActiveModel {
            color: Set(color.clone()),
            ..Default::default()
        };
        let id = self.id.get_id_cloned()?;

        let set_column = get_set_column(&active_model);

        entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
            .map_err(|f| {
                GenericSqlError::CannotUpdate(from_dberr_and_activemodel(set_column, &f))
            })?;

        self.color = color;

        Ok(())
    }
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), RoleError> {
        let model = entity::roles::ActiveModel::from(self.clone());

        let result = insert(model, executor).await?;

        self.id = Id::Id(result.last_insert_id);

        Ok(())
    }
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), RoleError> {
        let id = self.id.get_id_cloned()?;

        let Ok(Some(model)) = entity::roles::Entity::find_by_id(id)
            .one(executor)
            .await else {
                return Err(RoleError::CannotDelete);
            };

        let active_model: entity::roles::ActiveModel = model.into();

        delete(active_model, executor).await?;

        Ok(())
    }

    async fn add_user(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), RoleError> {
        let am = assignation::ActiveModel {
            role_id: Set(self.id.get_id_cloned()?),
            user_id: Set(userid.0.get_id_cloned()?),
            server_id: Set(self.server_id.id.clone()),
        };

        insert(am, executor).await.map(|_| ())?;

        Ok(())
    }
}
