use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emoji {
    id: i32,
    name: String,
    path: String,
}
