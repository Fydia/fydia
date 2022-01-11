use crate::{instance::Instance, server::Servers};
use fydia_utils::hash;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Default)]
pub struct User {
    pub id: UserId,
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
    pub servers: Servers,
}

impl User {
    pub fn new<T: Into<String>>(name: T, email: T, password: T, instance: Instance) -> User {
        User {
            id: UserId::new(0),
            name: name.into(),
            instance,
            token: None,
            description: None,
            password: Some(hash(password.into())),
            servers: Servers::new(),
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

    pub fn to_userinfo(&self) -> UserInfo {
        UserInfo::new(
            self.id.id,
            &self.name,
            &self.email,
            &self.description.clone().unwrap_or_default(),
            self.servers.clone(),
        )
    }

    pub fn take_value_of(&mut self, from: User) {
        self.id = from.id;
        self.name = from.name;
        self.instance = from.instance;
        self.token = from.token;
        self.email = from.email;
        self.password = from.password;
        self.description = from.description;
        self.servers = from.servers;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, PartialOrd)]
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

const HEADERNAME: &str = "token";

pub struct Token(pub String);

impl Token {
    pub fn from_headervalue(headers: &HeaderMap) -> Option<Token> {
        if let Some(token) = headers.get(HEADERNAME) {
            if let Ok(e) = token.to_str() {
                return Some(Token(e.to_string()));
            }
        };
        None
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    id: i32,
    name: String,
    email: String,
    description: String,
    servers: Servers,
}

impl UserInfo {
    pub fn new<T: Into<String>>(
        id: i32,
        name: T,
        email: T,
        description: T,
        servers: Servers,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            email: email.into(),
            description: description.into(),
            servers,
        }
    }
}
