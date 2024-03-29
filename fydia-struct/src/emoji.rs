//! This modules is related to emoji

use fydia_utils::serde::{Deserialize, Serialize};

/// `Emoji` is the struct that contains all information of an emoji
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct Emoji {
    id: i32,
    name: String,
    path: String,
}
