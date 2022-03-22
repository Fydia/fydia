use std::convert::TryFrom;

use crate::entity::user::ActiveModel as UserActiveModel;
use crate::entity::user::Entity as UserEntity;
use crate::entity::user::Model;
use crate::sqlpool::DbConnection;
use async_trait::async_trait;
use fydia_crypto::password::hash;
use fydia_crypto::password::verify_password;
use fydia_struct::user::Token;
use fydia_struct::user::UserId;
use fydia_struct::user::UserInfo;
use fydia_struct::{server::ServerId, user::User};
use fydia_utils::generate_string;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::Set;

#[async_trait]
pub trait SqlUser {
    async fn get_user_by_email_and_password<T: Into<String> + Send>(
        email: T,
        password: T,
        executor: &DatabaseConnection,
    ) -> Option<Self>
    where
        Self: Sized;
    async fn get_user_by_id(id: i32, executor: &DatabaseConnection) -> Option<Self>
    where
        Self: Sized;
    async fn get_user_by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self>
    where
        Self: Sized;
    async fn update_from_database(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_password<T: Into<String> + Send>(
        &mut self,
        clear_password: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    /// Prefere use [SqlServer::join()](`crate::impls::server::SqlServer::join()`)
    fn insert_server(&mut self, server_short_id: &ServerId) -> Result<(), String>;
    async fn insert_user(&mut self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn insert_user_and_update(&mut self, executor: &DatabaseConnection)
        -> Result<(), String>;
    async fn delete_account(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn get_user_message() -> Vec<String>;
}

#[async_trait]
impl SqlUser for User {
    async fn get_user_by_email_and_password<T: Into<String> + Send>(
        email: T,
        password: T,
        executor: &DatabaseConnection,
    ) -> Option<Self> {
        let model = Model::get_model_by(
            crate::entity::user::Column::Email.contains(email.into().as_str()),
            executor,
        )
        .await
        .ok()?;

        let password_is_good = verify_password(
            password.into().into(),
            std::borrow::Cow::Borrowed(&model.password),
        );

        if password_is_good {
            model.to_user(executor).await.ok()
        } else {
            None
        }
    }

    async fn get_user_by_id(id: i32, executor: &DatabaseConnection) -> Option<Self> {
        Model::get_model_by_id(&id, executor)
            .await
            .ok()?
            .to_user(executor)
            .await
            .ok()
    }

    async fn get_user_by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self> {
        Model::get_model_by_token(&token.0, executor)
            .await
            .ok()?
            .to_user(executor)
            .await
            .ok()
    }

    async fn update_from_database(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = Model::get_model_by_id(&self.id.0, executor).await?;
        let user_of_db = model.to_user(executor).await?;

        self.take_value_of(user_of_db);
        Ok(())
    }

    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let token = generate_string(30);
        let mut active_model: UserActiveModel =
            Model::get_model_by_id(&self.id.0, executor).await?.into();
        active_model.token = Set(token.clone());

        UserEntity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.token = Some(token.clone());
        Ok(())
    }

    async fn update_name<T: Into<String> + Send>(
        &mut self,
        name: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let name = name.into();
        let mut active_model: UserActiveModel =
            Model::get_model_by_id(&self.id.0, executor).await?.into();

        active_model.name = Set(name.clone());

        UserEntity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.name = name;
        Ok(())
    }

    async fn update_password<T: Into<String> + Send>(
        &mut self,
        clear_password: T,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let clear_password = clear_password.into();
        let mut active_model: UserActiveModel =
            Model::get_model_by_id(&self.id.0, executor).await?.into();

        let password = hash(clear_password)?;
        active_model.password = Set(password.clone());

        UserEntity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;
        self.password = Some(password);
        Ok(())
    }

    fn insert_server(&mut self, server_short_id: &ServerId) -> Result<(), String> {
        self.servers.0.push(server_short_id.clone());

        Ok(())
    }

    async fn insert_user(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let active_model: UserActiveModel = UserActiveModel::try_from(self.clone())?;

        let db = UserEntity::insert(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.id = UserId::new(db.last_insert_id);

        Ok(())
    }

    async fn insert_user_and_update(
        &mut self,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        self.insert_user(executor).await?;
        self.update_from_database(executor).await
    }

    async fn delete_account(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = Model::get_model_by_id(&self.id.0, executor).await?;
        let active_model: UserActiveModel = model.into();

        UserEntity::delete(active_model)
            .exec(executor)
            .await
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    async fn get_user_message() -> Vec<String> {
        Vec::new()
    }
}
#[async_trait]
pub trait UserFrom {
    async fn get_user(&self, executor: &DatabaseConnection) -> Option<User>;
}

#[async_trait]
impl UserFrom for UserId {
    async fn get_user(&self, executor: &DatabaseConnection) -> Option<User> {
        User::get_user_by_id(self.0, executor).await
    }
}

#[async_trait]
impl UserFrom for UserInfo {
    async fn get_user(&self, executor: &DatabaseConnection) -> Option<User> {
        User::get_user_by_id(self.id.0, executor).await
    }
}

#[async_trait]
pub trait UserInfoSql {
    async fn to_userinfo_from(&self, executor: DbConnection) -> Option<UserInfo>;
}
#[async_trait]
impl UserInfoSql for UserId {
    async fn to_userinfo_from(&self, executor: DbConnection) -> Option<UserInfo> {
        self.get_user(&executor).await.map(|f| f.to_userinfo())
    }
}
