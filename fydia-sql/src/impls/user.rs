use crate::entity::user::ActiveModel as UserActiveModel;
use crate::entity::user::Entity as UserEntity;
use async_trait::async_trait;
use fydia_struct::server::Servers;
use fydia_struct::user::Token;
use fydia_struct::{server::ServerId, user::User};
use fydia_utils::generate_string;
use fydia_utils::hash;
use fydia_utils::verify_password;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;

#[async_trait]
pub trait SqlUser {
    async fn get_user_by_email_and_password(
        email: String,
        password: String,
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
    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<String, String>;
    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn update_password(
        &mut self,
        clear_password: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn insert_server(
        &mut self,
        server_short_id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<(), String>;
    async fn insert_user(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn delete_account(&self, executor: &DatabaseConnection) -> Result<(), String>;
    async fn get_user_message() -> Vec<String>;
}

#[async_trait]
impl SqlUser for User {
    async fn get_user_by_email_and_password(
        email: String,
        password: String,
        executor: &DatabaseConnection,
    ) -> Option<Self> {
        match UserEntity::find()
            .filter(crate::entity::user::Column::Email.contains(email.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(model)) => {
                if verify_password(password, &model.password) {
                    model.to_user()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    async fn get_user_by_id(id: i32, executor: &DatabaseConnection) -> Option<Self> {
        match UserEntity::find_by_id(id).one(executor).await {
            Ok(Some(e)) => e.to_user(),
            Err(_) => None,
            _ => None,
        }
    }

    async fn get_user_by_token(token: &Token, executor: &DatabaseConnection) -> Option<Self> {
        match UserEntity::find()
            .filter(crate::entity::user::Column::Token.contains(token.0.as_str()))
            .one(executor)
            .await
        {
            Ok(Some(e)) => e.to_user(),
            Err(_) => None,
            _ => None,
        }
    }

    async fn update_token(&mut self, executor: &DatabaseConnection) -> Result<String, String> {
        let token = generate_string(30);
        match UserEntity::find_by_id(self.id).one(executor).await {
            Ok(Some(model)) => {
                let mut active_model: UserActiveModel = model.into();
                active_model.token = Set(token.clone());

                match UserEntity::update(active_model).exec(executor).await {
                    Ok(_) => {
                        self.token = Some(token.clone());
                        Ok(token)
                    }
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error message".to_string()),
        }
    }

    async fn update_name(
        &mut self,
        name: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match UserEntity::find_by_id(self.id).one(executor).await {
            Ok(Some(model)) => {
                let mut active_model: UserActiveModel = model.into();
                active_model.name = Set(name.clone());

                match UserEntity::update(active_model).exec(executor).await {
                    Ok(_) => {
                        self.name = name;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error message".to_string()),
        }
    }

    async fn update_password(
        &mut self,
        clear_password: String,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match UserEntity::find_by_id(self.id).one(executor).await {
            Ok(Some(model)) => {
                let password = hash(clear_password);
                let mut active_model: UserActiveModel = model.into();
                active_model.password = Set(password.clone());

                match UserEntity::update(active_model).exec(executor).await {
                    Ok(_) => {
                        self.password = password;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error message".to_string()),
        }
    }

    async fn insert_server(
        &mut self,
        server_short_id: ServerId,
        executor: &DatabaseConnection,
    ) -> Result<(), String> {
        match UserEntity::find_by_id(self.id).one(executor).await {
            Ok(Some(model)) => {
                self.server.0.push(server_short_id);
                let json = match serde_json::to_string(&self.server.0) {
                    Ok(json) => json,
                    Err(error) => error.to_string(),
                };
                let mut active_model: UserActiveModel = model.into();
                active_model.server = Set(Some(json));

                match UserEntity::update(active_model).exec(executor).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error message".to_string()),
        }
    }

    async fn insert_user(&self, executor: &DatabaseConnection) -> Result<(), String> {
        let json = match serde_json::to_string(&Servers(Vec::new())) {
            Ok(json) => json,
            Err(error) => return Err(error.to_string()),
        };
        let active_model = UserActiveModel {
            name: Set(self.name.clone()),
            token: Set("".to_string()),
            email: Set(self.email.clone()),
            password: Set(self.password.clone()),
            server: Set(Some(json)),
            ..Default::default()
        };

        match UserEntity::insert(active_model).exec(executor).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
        }
    }

    async fn delete_account(&self, executor: &DatabaseConnection) -> Result<(), String> {
        match UserEntity::find_by_id(self.id).one(executor).await {
            Ok(Some(model)) => {
                let active_model: UserActiveModel = model.into();
                match UserEntity::delete(active_model).exec(executor).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error");
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Error");
                return Err(e.to_string());
            }
            _ => Err("Cannot get error message".to_string()),
        }
    }

    async fn get_user_message() -> Vec<String> {
        Vec::new()
    }
}
