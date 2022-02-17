use serde::{Deserialize, Serialize};

/// `Emoji` is the struct that contains all information of an emoji
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emoji {
    id: i32,
    name: String,
    path: String,
}
