use async_trait::async_trait;
use entity::user::ActiveModel as UserActiveModel;
use entity::user::Column;
use entity::user::Entity as UserEntity;
use entity::user::Model;
use fydia_crypto::password::hash;
use fydia_crypto::password::verify_password;
use fydia_struct::user::Token;
use fydia_struct::user::User;
use fydia_struct::user::UserId;
use fydia_utils::async_trait;
use fydia_utils::generate_string;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::Set;
use std::convert::TryFrom;

use super::basic_model::BasicModel;
use super::delete;
use super::update;

#[async_trait]
pub trait SqlUser {
    async fn by_email_and_password(
        email: &str,
        password: &str,
        executor: &DatabaseConnection,
    ) -> Option<Self>
    where
        Self: Sized;
    async fn by_id(id: u32, executor: &DatabaseConnection) -> Option<Self>
    where
        Self: Sized;
    async fn by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self>
    where
        Self: Sized;
    async fn update_from_database(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name(
        &mut self,
        name: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_password(
        &mut self,
        clear_password: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn insert(mut self, executor: &DatabaseConnection) -> Result<User, String>;
    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), String>;
}

#[async_trait]
impl SqlUser for User {
    async fn by_email_and_password(
        email: &str,
        password: &str,
        executor: &DatabaseConnection,
    ) -> Option<Self> {
        let model = Model::get_model_by(&[entity::user::Column::Email.eq(email)], executor)
            .await
            .ok()?;

        let password_is_good =
            verify_password(password.into(), std::borrow::Cow::Borrowed(&model.password));

        if password_is_good {
            model.to_struct(executor).await.ok()
        } else {
            None
        }
    }

    async fn by_id(id: u32, executor: &DatabaseConnection) -> Option<Self> {
        Model::get_model_by(&[Column::Id.eq(id)], executor)
            .await
            .ok()?
            .to_struct(executor)
            .await
            .ok()
    }

    async fn by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self> {
        Model::get_model_by(&[Column::Token.eq(token.0.as_str())], executor)
            .await
            .ok()?
            .to_struct(executor)
            .await
            .ok()
    }

    async fn update_from_database(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model =
            Model::get_model_by(&[Column::Id.eq(self.id.0.get_id_cloned()?)], executor).await?;
        let user_of_db = model.to_struct(executor).await?;

        self.take_value_of(user_of_db);
        Ok(())
    }

    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let token = generate_string(30);
        let mut active_model: UserActiveModel =
            Model::get_model_by(&[Column::Id.eq(self.id.0.get_id_cloned()?)], executor)
                .await?
                .into();
        active_model.token = Set(token.clone());

        update(active_model, executor).await?;

        self.token = Some(token.clone());
        Ok(())
    }

    async fn update_name(
        &mut self,
        name: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let mut active_model: UserActiveModel =
            Model::get_model_by(&[Column::Id.eq(self.id.0.get_id_cloned()?)], executor)
                .await?
                .into();

        active_model.name = Set(name.to_string());

        update(active_model, executor).await?;

        self.name = name.to_string();
        Ok(())
    }

    async fn update_password(
        &mut self,
        clear_password: &str,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let mut active_model: UserActiveModel =
            Model::get_model_by(&[Column::Id.eq(self.id.0.get_id_cloned()?)], executor)
                .await?
                .into();

        let password = hash(clear_password)?;
        active_model.password = Set(password.clone());

        update(active_model, executor).await?;

        self.password = Some(password);
        Ok(())
    }

    async fn insert(mut self, executor: &DatabaseConnection) -> Result<Self, String> {
        if self.token.is_none() {
            self.token = Some(generate_string(30));
        }

        let active_model: UserActiveModel = UserActiveModel::try_from(self.clone())?;
        let db = UserEntity::insert(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.id = UserId::new(db.last_insert_id);

        Ok(self)
    }

    async fn delete(mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model =
            Model::get_model_by(&[Column::Id.eq(self.id.0.get_id_cloned()?)], executor).await?;
        let active_model: UserActiveModel = model.clone().into();

        delete(active_model, executor).await?;

        self = Self::default();

        Ok(())
    }
}
#[async_trait]
pub trait UserFrom {
    async fn to_user(&self, executor: &DatabaseConnection) -> Option<User>;
}

#[async_trait]
impl UserFrom for UserId {
    async fn to_user(&self, executor: &DatabaseConnection) -> Option<User> {
        User::by_id(self.0.get_id_cloned().ok()?, executor).await
    }
}
