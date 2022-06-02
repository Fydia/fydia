//! `DirectMessage`

use fydia_utils::serde::Serialize;

use crate::utils::Id;

/// `DirectMessage` is the struct that reprensent a direct message
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct DirectMessage {
    pub id: Id<u32>,
    pub name: String,
    pub icons: String,
}

impl DirectMessage {
    /// Create a new `DirectMessage` from arguments
    pub fn new(id: Id<u32>, name: String, icons: String) -> Self {
        Self { id, name, icons }
    }
}
