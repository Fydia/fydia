use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum Permission {
    Write = 2,
    Read = 1,
    None = 0,
}

impl Permission {
    pub fn from_string(from: String) -> Self {
        match from.to_ascii_uppercase().as_str() {
            "WRITE" => Permission::Write,
            "READ" => Permission::Read,
            _ => Permission::None,
        }
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Write => write!(f, "WRITE"),
            Permission::Read => write!(f, "READ"),
            Permission::None => write!(f, "NONE"),
        }
    }
}
