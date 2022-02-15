use crate::entity::user::ActiveModel as UserActiveModel;
use crate::entity::user::Entity as UserEntity;
use crate::sqlpool::DbConnection;
use async_trait::async_trait;
use fydia_crypto::password::hash;
use fydia_crypto::password::verify_password;
use fydia_struct::server::Servers;
use fydia_struct::user::Token;
use fydia_struct::user::UserId;
use fydia_struct::user::UserInfo;
use fydia_struct::{server::ServerId, user::User};
use fydia_utils::generate_string;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
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
    async fn insert_server(
        &mut self,
        server_short_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
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
        let model = UserEntity::find()
            .filter(crate::entity::user::Column::Email.contains(email.into().as_str()))
            .one(executor)
            .await
            .ok()??;
        let password_is_good = verify_password(
            password.into().into(),
            std::borrow::Cow::Borrowed(&model.password),
        );

        if password_is_good {
            model.to_user()
        } else {
            None
        }
    }

    async fn get_user_by_id(id: i32, executor: &DatabaseConnection) -> Option<Self> {
        UserEntity::find_by_id(id)
            .one(executor)
            .await
            .ok()??
            .to_user()
    }

    async fn get_user_by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self> {
        UserEntity::find()
            .filter(crate::entity::user::Column::Token.contains(token.0.as_str()))
            .one(executor)
            .await
            .ok()??
            .to_user()
    }

    async fn update_from_database(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "No User".to_string())?;

        let user_of_db = model
            .to_user()
            .ok_or_else(|| "Can't convert it to user".to_string())?;

        self.take_value_of(user_of_db);
        Ok(())
    }

    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let token = generate_string(30);
        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "No User".to_string())?;

        let mut active_model: UserActiveModel = model.into();
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
        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "No User".to_string())?;

        let mut active_model: UserActiveModel = model.into();
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

        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "No User".to_string())?;

        let password = hash(clear_password)?;
        let mut active_model: UserActiveModel = model.into();
        active_model.password = Set(password.clone());

        UserEntity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;
        self.password = Some(password);
        Ok(())
    }

    async fn insert_server(
        &mut self,
        server_short_id: &ServerId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or_else(|| "No User".to_string())?;

        let mut current_server = self.servers.clone().0;
        current_server.push(server_short_id.clone());

        let json = serde_json::to_string(&current_server).map_err(|f| f.to_string())?;

        let mut active_model: UserActiveModel = model.into();
        active_model.server = Set(Some(json));
        UserEntity::update(active_model)
            .exec(executor)
            .await
            .map_err(|f| f.to_string())?;

        self.servers.0.push(server_short_id.clone());

        Ok(())
    }

    async fn insert_user(&mut self, executor: &DatabaseConnection) -> Result<(), String> {
        let json = serde_json::to_string(&Servers(Vec::new())).map_err(|f| f.to_string())?;
        let password = self
            .password
            .clone()
            .ok_or_else(|| "Password is empty".to_string())?;

        let active_model = UserActiveModel {
            name: Set(self.name.clone()),
            token: Set("".to_string()),
            email: Set(self.email.clone()),
            password: Set(password),
            server: Set(Some(json)),
            ..Default::default()
        };

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
        let model = UserEntity::find_by_id(self.id.id)
            .one(executor)
            .await
            .map_err(|f| f.to_string())?
            .ok_or("No User")?;

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
pub trait UserIdSql {
    async fn get_user(&self, executor: &DatabaseConnection) -> Option<User>;
}

#[async_trait]
impl UserIdSql for UserId {
    async fn get_user(&self, executor: &DatabaseConnection) -> Option<User> {
        User::get_user_by_id(self.id, executor).await
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
