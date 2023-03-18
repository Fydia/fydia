//! This module is related to permission

use fydia_utils::serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    channel::{ChannelError, ChannelId},
    roles::{RoleError, RoleId},
    sqlerror::GenericSqlError,
    user::{UserError, UserId},
    utils::IdError,
};

/// Wrapper of `Vec<Permission>`
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct Permissions(Vec<Permission>);

impl Permissions {
    /// Create a new `Permissions`
    pub fn new(perms: Vec<Permission>) -> Self {
        Self(perms)
    }

    /// Return true if there is no permission
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return permissions
    pub fn get(self) -> Vec<Permission> {
        self.0
    }

    /// Return true if user can read
    pub fn can_read(&self) -> bool {
        if let Ok(value) = self.calculate(None) {
            return value.can_read();
        }

        false
    }

    /// Return true if user can write
    pub fn can_write(&self) -> bool {
        if let Ok(value) = self.calculate(None) {
            return value.can_write();
        }

        false
    }

    /// Return true if user is admin
    pub fn is_admin(&self) -> bool {
        if let Ok(value) = self.calculate(None) {
            return value.can_write();
        }

        false
    }

    /// Return true if user can do this
    pub fn can(&self, pvalue: &PermissionValue) -> bool {
        for i in &self.0 {
            if !i.can(pvalue) {
                return false;
            }
        }

        true
    }

    /// Take multiple permissions and return one
    ///
    /// User permisison is privileged to a role permission.
    ///
    /// # Errors
    /// Return an error if :
    /// * there is no `Permission` have `PermissionType::User` as type
    pub fn calculate(&self, channelid: Option<ChannelId>) -> Result<Permission, PermissionError> {
        if let Some(user_perm) = self
            .0
            .iter()
            .find(|i| matches!(i.permission_type, PermissionType::User(_)))
        {
            return Ok(user_perm.clone());
        }

        let mut value = 0;
        for i in &self.0 {
            if let PermissionType::Role(_) = i.permission_type {
                value |= i.value;
            }
        }

        let channelid = channelid.ok_or_else(|| PermissionError::NoChannelId)?;

        Ok(Permission {
            permission_type: PermissionType::Channel(channelid.clone()),
            channelid: Some(channelid),
            value,
        })
    }
}

#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Permission {
    #[serde(flatten)]
    pub permission_type: PermissionType,
    pub channelid: Option<ChannelId>,
    pub value: u64,
}

impl Permission {
    /// Create a new Role permission
    pub fn role(role: RoleId, channelid: Option<ChannelId>, value: u64) -> Self {
        Self {
            permission_type: PermissionType::Role(role),
            channelid,
            value,
        }
    }
    /// Create a new User permission
    pub fn user(userid: UserId, channelid: Option<ChannelId>, value: u64) -> Self {
        Self {
            permission_type: PermissionType::User(userid),
            channelid,
            value,
        }
    }

    /// Return true if user can read
    pub fn can_read(&self) -> bool {
        self.can(&PermissionValue::Read)
    }

    /// Return true if user can write
    pub fn can_write(&self) -> bool {
        self.can(&PermissionValue::Write)
    }

    /// Return true if user is admin
    pub fn is_admin(&self) -> bool {
        self.can(&PermissionValue::Admin)
    }

    /// Return true if user can do the `PermissionValue`
    fn can(&self, pvalue: &PermissionValue) -> bool {
        let perm = pvalue.to_u64();
        self.value & perm == perm
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self {
            permission_type: PermissionType::User(UserId::default()),
            channelid: Option::default(),
            value: Default::default(),
        }
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `PermissionError` represents all errors of `Permission`
pub enum PermissionError {
    #[error("No permission for this channel and this role")]
    CannotGetByChannelAndRole,
    #[error("No permission for this channel and this user")]
    CannotGetByChannelAndUser,
    #[error("No permission for this channel")]
    CannotGetByChannel,
    #[error("Permission have a bad type")]
    PermissionTypeError,
    #[error("Cannot convert the model to struct")]
    ModelToStruct,
    #[error("No id for this channel")]
    NoChannelId,
    #[error("No role id")]
    NoRoleId,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
    #[error("{0}")]
    CannotGetRoles(Box<RoleError>),
    #[error("{0}")]
    UserError(Box<UserError>),
    #[error("{0}")]
    ChannelError(Box<ChannelError>),
    #[error("{0}")]
    Other(String),
}

impl From<IdError> for PermissionError {
    fn from(value: IdError) -> Self {
        match value {
            IdError::IdUnset => PermissionError::NoRoleId,
        }
    }
}

impl From<RoleError> for PermissionError {
    fn from(value: RoleError) -> Self {
        Self::CannotGetRoles(Box::new(value))
    }
}

impl From<UserError> for PermissionError {
    fn from(value: UserError) -> Self {
        Self::UserError(Box::new(value))
    }
}

impl From<ChannelError> for PermissionError {
    fn from(value: ChannelError) -> Self {
        Self::ChannelError(Box::new(value))
    }
}

impl From<GenericSqlError> for PermissionError {
    fn from(value: GenericSqlError) -> Self {
        Self::GenericSqlError(Box::new(value))
    }
}

#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub enum PermissionType {
    #[serde(rename = "role")]
    Role(RoleId),
    #[serde(rename = "user")]
    User(UserId),
    #[serde(skip)]
    Channel(ChannelId),
}

/// `Permission` contains all permission as enum
#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(crate = "fydia_utils::serde")]
pub enum PermissionValue {
    Admin = (1 << 2),
    Write = (1 << 1),
    Read = (1 << 0),
    None = 0,
}

impl PermissionValue {
    fn to_u64(&self) -> u64 {
        self.clone() as u64
    }
}
