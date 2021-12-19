use crate::{instance::Instance, server::Servers};
use fydia_utils::hash;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Default)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub instance: Instance,
    #[serde(skip)]
    pub token: Option<String>,
    #[serde(skip)]
    pub email: String,
    #[serde(skip)]
    pub password: Option<String>,
    #[serde(skip)]
    pub description: Option<String>,
    #[serde(skip)]
    pub server: Servers,
}

impl User {
    pub fn new<T: Into<String>>(name: T, email: T, password: T, instance: Instance) -> User {
        User {
            id: 0,
            name: name.into(),
            instance,
            token: None,
            description: None,
            password: Some(hash(password.into())),
            server: Servers::new(),
            email: email.into(),
        }
    }
    pub fn drop_token(&mut self) {
        self.token = None;
    }
    pub fn drop_password(&mut self) {
        self.password = None;
    }

    pub fn drop_sensitive_information(&mut self) {
        self.drop_token();
        self.drop_password();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId {
    pub id: i32,
}

impl UserId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.to_string()),
        }
    }
}

pub struct Token(pub String);

impl Token {
    pub fn from_headervalue(headers: &HeaderMap) -> Option<Token> {
        if let Some(token) = headers.get("token") {
            if let Ok(e) = token.to_str() {
                return Some(Token(e.to_string()));
            }
        };
        None
    }
}
