//! This module is related to permission

use fydia_utils::serde::{Deserialize, Serialize};

use crate::{channel::ChannelId, roles::RoleId, user::UserId};

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

    /// Return true if user can do this
    pub fn can(&self, pvalue: &PermissionValue) -> bool {
        for i in &self.0 {
            if !i.can(pvalue) {
                return false;
            }
        }

        true
    }

    /// Return true if user can do this
    pub fn can_vec(&self, pvalues: &[PermissionValue]) -> bool {
        for i in &self.0 {
            if !i.can_vec(pvalues) {
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
    pub fn calculate(&self, channelid: Option<ChannelId>) -> Result<Permission, String> {
        if let Some(user_perm) = self
            .0
            .iter()
            .filter(|i| {
                if let PermissionType::User(_) = i.permission_type {
                    true
                } else {
                    false
                }
            })
            .nth(0)
        {
            return Ok(user_perm.clone());
        }

        let mut value = 0;
        for i in self.0.iter() {
            match i.permission_type {
                PermissionType::Role(_) => {
                    value |= i.value;
                }
                _ => {}
            }
        }
        let channelid = channelid.ok_or_else(|| "No channelid".to_string())?;

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

    /// Return true if user can do the `PermissionValue`
    pub fn can(&self, pvalue: &PermissionValue) -> bool {
        let perm = pvalue.to_u64();
        self.value & perm == perm
    }

    /// Return true if user can do all of the `PermissionValue`
    pub fn can_vec(&self, pvalues: &[PermissionValue]) -> bool {
        let can = true;
        for pvalue in pvalues {
            let perm = pvalue.to_u64();
            if can && self.value & perm != perm {
                return false;
            }
        }

        can
    }
}

impl Default for Permission {
    fn default() -> Self {
        Self {
            permission_type: PermissionType::User(Default::default()),
            channelid: Default::default(),
            value: Default::default(),
        }
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
