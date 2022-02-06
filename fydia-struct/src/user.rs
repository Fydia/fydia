use crate::{instance::Instance, server::Servers};
use fydia_crypto::password::hash;
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
    pub fn new<T: Into<String>>(
        name: T,
        email: T,
        password: T,
        instance: Instance,
    ) -> Result<User, String> {
        let name = name.into();
        let email = email.into();
        let password = password.into();
        if name.is_empty() {
            return Err("Name is empty".to_string());
        }

        if email.is_empty() {
            return Err("Email is empty".to_string());
        }

        if password.is_empty() {
            return Err("Password is empty".to_string());
        }

        Ok(User {
            name,
            instance,
            password: hash(password).ok(),
            email,
            ..Default::default()
        })
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct UserId {
    pub id: i32,
}

impl Default for UserId {
    fn default() -> Self {
        Self { id: -1 }
    }
}

impl UserId {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
    pub fn to_string(&self) -> Result<String, String> {
        serde_json::to_string(&self).map_err(|f| f.to_string())
    }
}

pub const HEADERNAME: &str = "Authorization";

pub struct Token(pub String);

impl Token {
    pub fn from_headervalue(headers: &HeaderMap) -> Option<Token> {
        let token = headers.get(HEADERNAME)?;

        Some(Token(token.to_str().ok()?.to_string()))
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
