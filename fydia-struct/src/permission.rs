use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum Permission {
    Write = 2,
    Read = 1,
    None = 0,
}

impl Permission {
    pub fn to_string(&self) -> String {
        match self {
            Permission::Write => "WRITE".to_string(),
            Permission::Read => "READ".to_string(),
            Permission::None => "NONE".to_string(),
        }
    }

    pub fn from_string(from: String) -> Self {
        match from.to_ascii_uppercase().as_str() {
            "WRITE" => Permission::Write,
            "READ" => Permission::Read,
            _ => Permission::None,
        }
    }
}
