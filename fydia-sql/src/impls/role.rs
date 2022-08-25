use entity::roles::assignation;
use fydia_struct::{
    response::FydiaResponse, roles::Role, server::ServerId, user::UserId, utils::Id,
};

use super::{delete, insert};
use fydia_utils::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[async_trait::async_trait]
pub trait SqlRoles {
    async fn by_server_id<'r>(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Role>, FydiaResponse<'r>>;
    async fn by_id<'a>(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Role, FydiaResponse<'a>>;
    async fn insert<'a>(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>>;
    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>>;
    async fn delete<'a>(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>>;
    async fn add_user<'a>(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>>;
}

#[async_trait::async_trait]
impl SqlRoles for Role {
    async fn by_server_id<'r>(
        shortid: &str,
        executor: &DatabaseConnection,
    ) -> Result<Vec<Self>, FydiaResponse<'r>> {
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

    async fn by_id<'a>(
        role_id: u32,
        shortid: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<Self, FydiaResponse<'a>> {
        let query = entity::roles::Entity::find()
            .filter(entity::roles::Column::Id.eq(role_id))
            .filter(entity::roles::Column::Serverid.eq(shortid.id.as_str()))
            .one(executor)
            .await;
        match query {
            Ok(Some(model)) => Ok(model.to_role()),
            Err(e) => Err(FydiaResponse::StringError(e.to_string())),
            _ => Err(FydiaResponse::TextError("No Role with this id")),
        }
    }

    async fn update_name<'a, T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>> {
        let name = name.into();
        let active_model = entity::roles::ActiveModel {
            name: Set(name.clone()),
            ..Default::default()
        };
        let id = self
            .id
            .get_id_cloned()
            .map_err(FydiaResponse::StringError)?;

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.name = name;
                Ok(())
            }
            Err(e) => Err(FydiaResponse::StringError(e.to_string())),
        }
    }

    async fn update_color<'a, T: Into<String> + Send>(
        &mut self,
        color: T,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>> {
        let color = color.into();
        let active_model = entity::roles::ActiveModel {
            color: Set(color.clone()),
            ..Default::default()
        };
        let id = self
            .id
            .get_id_cloned()
            .map_err(FydiaResponse::StringError)?;

        match entity::roles::Entity::update(active_model)
            .filter(entity::messages::Column::Id.eq(id))
            .exec(executor)
            .await
        {
            Ok(_) => {
                self.color = color;
                Ok(())
            }
            Err(e) => Err(FydiaResponse::StringError(e.to_string())),
        }
    }
    async fn insert<'a>(&mut self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        let model = entity::roles::ActiveModel::from(self.clone());

        let result = insert(model, executor).await?;

        self.id = Id::Id(result.last_insert_id);

        Ok(())
    }
    async fn delete<'a>(&self, executor: &DatabaseConnection) -> Result<(), FydiaResponse<'a>> {
        let id = self
            .id
            .get_id_cloned()
            .map_err(FydiaResponse::StringError)?;

        let model = entity::roles::Entity::find_by_id(id)
            .one(executor)
            .await
            .map_err(|f| FydiaResponse::StringError(f.to_string()))?
            .ok_or(FydiaResponse::TextError("Can't the role"))?;

        let active_model: entity::roles::ActiveModel = model.into();

        delete(active_model, executor).await
    }

    async fn add_user<'a>(
        &self,
        userid: &UserId,
        executor: &DatabaseConnection,
    ) -> Result<(), FydiaResponse<'a>> {
        let am = assignation::ActiveModel {
            role_id: Set(self
                .id
                .get_id_cloned()
                .map_err(FydiaResponse::StringError)?),
            user_id: Set(userid
                .0
                .get_id_cloned()
                .map_err(FydiaResponse::StringError)?),
            server_id: Set(self.server_id.id.clone()),
        };

        insert(am, executor).await.map(|_| ())
    }
}
