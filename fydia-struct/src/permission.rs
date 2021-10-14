use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum Permission {
    Admin = (1 << 2),
    Write = (1 << 1),
    Read = (1 << 0),
    NoPerm = 0,
}

impl Permission {
    pub fn from_string<T: Into<String>>(from: T) -> Self {
        let from: String = from.into();
        match from.to_ascii_uppercase().as_str() {
            "WRITE" => Permission::Write,
            "READ" => Permission::Read,
            _ => Permission::NoPerm,
        }
    }

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
