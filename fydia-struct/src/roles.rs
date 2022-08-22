//! This module is related to roles

use crate::{server::ServerId, utils::Id};
use fydia_utils::serde::{Deserialize, Serialize};

/// Id of Role
pub type RoleId = Id<u32>;
/// `Role` contains all value of roles
#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Role {
    pub id: RoleId,
    pub server_id: ServerId,
    pub name: String,
    pub color: String,
    pub server_permission: u64,
}
