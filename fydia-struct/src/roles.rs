//! This module is related to roles

use crate::{
    server::ServerId,
    sqlerror::{GenericError, GenericSqlError},
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

impl From<GenericSqlError> for RoleError {
    fn from(value: GenericSqlError) -> Self {
        match value {
            GenericSqlError::CannotInsert(_) => Self::CannotAddUser,
            GenericSqlError::CannotUpdate(GenericError { set_column, error }) => {
                error!("{error}");

                if set_column.contains(&"color".to_string()) {
                    return Self::CannotUpdateColor;
                }

                Self::CannotUpdateName
            }
            GenericSqlError::CannotDelete(_) => Self::CannotDelete,
        }
    }
}
