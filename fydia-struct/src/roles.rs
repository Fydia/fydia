use crate::permission::Permission;
use crate::server::ServerId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Role {
    pub id: i32,
    pub server_id: ServerId,
    pub name: String,
    pub color: String,
    pub channel_access: ChannelAccess,
    pub permission: Permission,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ChannelAccess(pub Vec<String>);

impl ChannelAccess {
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}