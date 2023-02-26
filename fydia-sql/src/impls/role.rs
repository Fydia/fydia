use entity::roles::assignation;
use fydia_struct::{
    response::{FydiaResponse, IntoFydia, MapError},
    roles::Role,
    server::ServerId,
    user::UserId,
    utils::Id,
};

use super::{delete, insert};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use shared::sea_orm;
#[async_trait::async_trait]
pub trait SqlRoles {
    async fn by_server_id(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, FydiaResponse>;
    async fn by_id(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Role, FydiaResponse>;
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse>;
    async fn add_user(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn by_server_id(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, FydiaResponse> {
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
    ) -> Result<Self, FydiaResponse> {
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(role_id))
            .filter(entity::roles::Column::Serverid.eq(shortid.id.as_str()))
            .one(executor)
            .await;
        match query {
            Ok(Some(model)) => Ok(model.to_role()),
            Err(e) => Err(e.to_string().into_error()),
            _ => Err("No Role with this id".into_error()),
        }
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let name = name.into();
        let active_model = entity::roles::ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        };
        let id = self.id.get_id_cloned_fydiaresponse()?;

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.name = name;
                Ok(())
            }
            Err(e) => Err(e.to_string().into_error()),
        }
    }

    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let color = color.into();
        let active_model = entity::roles::ActiveModel {
            color: Set(color.clone()),
            ..Default::default()
        };
        let id = self.id.get_id_cloned_fydiaresponse()?;

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.color = color;
                Ok(())
            }
            Err(e) => Err(e.to_string().into_error()),
        }
    }
    async fn insert(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let model = entity::roles::ActiveModel::from(self.clone());

        let result = insert(model, executor).await?;

        self.id = Id::Id(result.last_insert_id);

        Ok(())
    }
    async fn delete(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse> {
        let id = self.id.get_id_cloned_fydiaresponse()?;

        let model = entity::roles::Entity::find_by_id(id)
            .one(executor)
            .await
            .error_to_fydiaresponse()?
            .ok_or_else(|| "Can't the role".into_error())?;

        let active_model: entity::roles::ActiveModel = model.into();

        delete(active_model, executor).await
    }

    async fn add_user(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse> {
        let am = assignation::ActiveModel {
            role_id: Set(self.id.get_id_cloned_fydiaresponse()?),
            user_id: Set(userid.0.get_id_cloned_fydiaresponse()?),
            server_id: Set(self.server_id.id.clone()),
        };

        insert(am, executor).await.map(|_| ())
    }
}
