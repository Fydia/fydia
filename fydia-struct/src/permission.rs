//! This module is related to permission

use std::fmt::Display;

use fydia_utils::serde::{Deserialize, Serialize};

use crate::{channel::ChannelId, roles::Role, user::UserId};

#[derive(Debug)]
pub struct Permissions(Vec<Permission>);

impl Permissions {
    pub fn new(perms: Vec<Permission>) -> Self {
        Self(perms)
    }

    pub fn can(&self, pvalue: &PermissionValue) -> bool {
        for i in &self.0 {
            if !i.can(&pvalue) {
                return false;
            }
        }

        true
    }

    pub fn can_vec(&self, pvalues: &[PermissionValue]) -> bool {
        for i in &self.0 {
            if !i.can_vec(pvalues) {
                return false;
            }
        }

        true
    }
}

#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct Permission {
    pub permission_type: PermissionType,
    pub channelid: ChannelId,
    pub value: u64,
}

impl Permission {
    pub fn role(role: Role, channelid: ChannelId, value: u64) -> Self {
        Self {
            permission_type: PermissionType::Role(role),
            channelid,
            value,
        }
    }

    pub fn user(userid: UserId, channelid: ChannelId, value: u64) -> Self {
        Self {
            permission_type: PermissionType::User(userid),
            channelid,
            value,
        }
    }

    pub fn can(&self, pvalue: &PermissionValue) -> bool {
        let perm = pvalue.to_u64();
        self.value & perm == perm
    }

    pub fn can_vec(&self, pvalues: &[PermissionValue]) -> bool {
        let can = true;
        for pvalue in pvalues {
            println!("{:?}", pvalue);
            let perm = pvalue.to_u64();
            if can && !(self.value & perm == perm) {
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
    Role(Role),
    User(UserId),
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
    /// Take a u64 represent Permission and a `Permission` to test*
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::permission::Permission;
    ///
    /// let perms: u32 = Permission::Read as u32 | Permission::Write as u32;
    ///
    /// // Permission::can will return true
    /// assert!(Permission::can(perms, Permission::Read));
    /// ```
    fn can(perms: u64, perm: PermissionValue) -> bool {
        let perm = perm as u64;
        perms & perm == perm
    }

    fn to_u64(&self) -> u64 {
        return self.clone() as u64;
    }
}
