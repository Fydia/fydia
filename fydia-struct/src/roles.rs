//! This module is related to roles

use crate::permission::Permission;
use crate::server::ServerId;
use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json,
};

/// `Role` contains all value of roles
#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Role {
    pub id: i32,
    pub server_id: ServerId,
    pub name: String,
    pub color: String,
}
