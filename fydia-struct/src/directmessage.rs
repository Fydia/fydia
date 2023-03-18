//! `DirectMessage`
use fydia_utils::serde::Serialize;
use thiserror::Error;

use crate::{
    sqlerror::GenericSqlError,
    user::UserError,
    utils::{Id, IdError},
};

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

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `DirectMessageError` represents all errors of `DirectMessage`
pub enum DirectMessageError {
    #[error("No DirectMessage with id")]
    CannotGetById,
    #[error("No DirectMessage with this user")]
    CannotGetByUser,
    #[error("No DirectMessage with members")]
    CannotGetMembers,
    #[error("Cannot add in dm")]
    CannotAdd,
    #[error("Cannot convert the model to struct")]
    ModelToStruct,
    #[error("User is not in dm")]
    UserNotInDm,
    #[error("{0}")]
    UserError(Box<UserError>),
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
}

impl From<IdError> for DirectMessageError {
    fn from(_: IdError) -> Self {
        Self::CannotGetById
    }
}

impl From<UserError> for DirectMessageError {
    fn from(value: UserError) -> Self {
        DirectMessageError::UserError(Box::new(value))
    }
}

impl From<GenericSqlError> for DirectMessageError {
    fn from(value: GenericSqlError) -> Self {
        DirectMessageError::GenericSqlError(Box::new(value))
    }
}
