//! This module is related to roles

use crate::permission::Permission;
use crate::server::ServerId;
use serde::{Deserialize, Serialize};

/// `Role` contains all value of roles
#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Role {
    pub id: i32,
    pub server_id: ServerId,
    pub name: String,
    pub color: String,
    pub channel_access: ChannelAccess,
    pub permission: Permission,
}

/// `ChannelAccess` contains all channel can be accessed by a `Role`
#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct ChannelAccess(pub Vec<String>);

impl ChannelAccess {
    /// Serialize `ChannelAccess` as Json and return a Result<String, Error>
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}
