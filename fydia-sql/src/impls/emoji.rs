use crate::sqlpool::FydiaPool;
use fydia_struct::emoji::Emoji;

#[async_trait::async_trait]
pub trait SqlEmoji {
    async fn get_emoji_by_server_id(server_id: String, executor: &FydiaPool) -> Vec<Emoji>;
}

#[async_trait::async_trait]
impl SqlEmoji for Emoji {
    async fn get_emoji_by_server_id(_server_id: String, _executor: &FydiaPool) -> Vec<Emoji> {
        Vec::new()
    }
}
