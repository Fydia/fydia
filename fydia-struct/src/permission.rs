//! This module is related to permission

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// `Permission` contains all permission as enum
#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum Permission {
    Admin = (1 << 2),
    Write = (1 << 1),
    Read = (1 << 0),
    NoPerm = 0,
}

impl Permission {
    /// Take a `Into<String>` value and return `Permission`
    ///
    /// Default value is `Permission::NoPerm`
    /// 
    /// # Examples
    ///
    /// ```
    /// use fydia_struct::permission::Permission;
    ///
    /// let perm = Permission::from_string("WRITE");
    ///
    /// assert_eq!(perm, Permission::Write);
    /// ```
    pub fn from_string<T: Into<String>>(from: T) -> Self {
        let from: String = from.into();
        match from.to_ascii_uppercase().as_str() {
            "WRITE" => Permission::Write,
            "READ" => Permission::Read,
            _ => Permission::NoPerm,
        }
    }

    /// Take a u32 represent Permission and a `Permission` to test*
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
    pub fn can(perms: u32, perm: Permission) -> bool {
        let perm_as_u32 = perm as u32;
        perms & perm_as_u32 == perm_as_u32
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Admin => write!(f, "ADMIN"),
            Permission::Write => write!(f, "WRITE"),
            Permission::Read => write!(f, "READ"),
            Permission::NoPerm => write!(f, "NOPERM"),
        }
    }
}
