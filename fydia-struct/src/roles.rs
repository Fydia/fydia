//! This module is related to roles

use crate::{
    server::ServerId,
    utils::{Id, IdError},
};
use fydia_utils::serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `RoleError` represents all errors of `Role`
pub enum RoleError {
    #[error("No role with this id")]
    NoRoleWithId,
    #[error("Cannot update name of the role")]
    CannotUpdateName,
    #[error("Cannot update cole of the role")]
    CannotUpdateColor,
    #[error("Cannot insert the role")]
    CannotInsert,
    #[error("Cannot delete the role")]
    CannotDelete,
    #[error("Cannot add this user to role")]
    CannotAddUser,
}

impl From<IdError> for RoleError {
    fn from(_: IdError) -> Self {
        Self::NoRoleWithId
    }
}
