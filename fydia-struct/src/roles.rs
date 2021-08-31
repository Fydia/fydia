use crate::permission::Permission;
use crate::server::ServerId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Role {
    pub id: i32,
    pub server_id: ServerId,
    pub name: String,
    pub color: String,
    pub channel_access: Vec<String>,
    pub permission: Permission,
}
