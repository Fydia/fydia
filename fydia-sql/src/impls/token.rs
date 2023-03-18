use super::user::SqlUser;
use fydia_struct::user::{Token, User, UserError};
use fydia_utils::async_trait;
use sea_orm::DatabaseConnection;
use shared::sea_orm;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait SqlToken {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Result<User, UserError>;
}

#[async_trait::async_trait]
impl SqlToken for Token {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Result<User, UserError> {
        User::by_token(self, executor).await
    }
}
