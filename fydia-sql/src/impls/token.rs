use fydia_struct::user::{Token, User};

use crate::sqlpool::FydiaPool;

use super::user::SqlUser;

#[async_trait::async_trait]
pub trait SqlToken {
    async fn get_user(&self, executor: &FydiaPool) -> Option<User>;
}

#[async_trait::async_trait]
impl SqlToken for Token {
    async fn get_user(&self, executor: &FydiaPool) -> Option<User> {
        User::get_user_by_token(self, executor).await
    }
}
