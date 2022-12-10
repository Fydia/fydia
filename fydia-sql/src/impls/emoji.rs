use fydia_struct::emoji::Emoji;
use fydia_utils::async_trait;
use sea_orm::DatabaseConnection;
use shared::sea_orm;
#[async_trait::async_trait]
pub trait SqlEmoji {
    async fn get_emoji_by_server_id(server_id: String, executor: &DatabaseConnection)
        -> Vec<Emoji>;
}

#[async_trait::async_trait]
impl SqlEmoji for Emoji {
    async fn get_emoji_by_server_id(
        _server_id: String,
        _executor: &DatabaseConnection,
    ) -> Vec<Emoji> {
        Vec::new()
    }
}
