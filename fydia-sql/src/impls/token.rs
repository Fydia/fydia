use fydia_struct::user::{Token, User};
use fydia_utils::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use super::user::SqlUser;

#[async_trait::async_trait]
pub trait SqlToken {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Option<User>;
}

#[async_trait::async_trait]
impl SqlToken for Token {
    async fn get_user(&self, executor: &Arc<DatabaseConnection>) -> Option<User> {
        User::by_token(self, executor).await
    }
}
