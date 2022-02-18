//! This module is related to user

use crate::{instance::Instance, server::Servers};
use fydia_crypto::password::hash;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

/// `User` contains all value of user
#[allow(missing_docs)]
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
    /// Return `User` if all given value are correct
    /// 
    /// # Errors
    /// If name or email or password is empty, an error will be return
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

    /// Remove token from `User`
    #[deprecated(note = "Please use UserInfo")]
    pub fn drop_token(&mut self) {
        self.token = None;
    }

    /// Remove password from `User`
    #[deprecated(note = "Please use UserInfo")]
    pub fn drop_password(&mut self) {
        self.password = None;
    }

    /// Remove password and token from `User`
    #[deprecated(note = "Please use UserInfo")]
    pub fn drop_sensitive_information(&mut self) {
        self.drop_token();
        self.drop_password();
    }

    /// Convert `User` to `UserInfo`
    pub fn to_userinfo(&self) -> UserInfo {
        UserInfo::new(
            self.id.id,
            &self.name,
            &self.email,
            &self.description.clone().unwrap_or_default(),
            self.servers.clone(),
        )
    }
    
    /// Clone `User` from another `User`
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

/// `UserId` contains id of `User`
#[allow(missing_docs)]
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
    /// Return a new `UserId`
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    /// Serialize UserId as Json and return `Ok(String)` if can be serialize
    /// or `Error(String)` if cannot
    pub fn to_string(&self) -> Result<String, String> {
        serde_json::to_string(&self).map_err(|f| f.to_string())
    }
}

/// Header name for the user token
pub const HEADERNAME: &str = "Authorization";

/// `Token` contains the token of a `User`
#[derive(Debug)]
pub struct Token(pub String);

impl Token {
    /// Return a Token from HTTP headers
    pub fn from_headervalue(headers: &HeaderMap) -> Option<Token> {
        let token = headers.get(HEADERNAME)?;

        Some(Token(token.to_str().ok()?.to_string()))
    }
}

/// `UserInfo` is `User` without sensitive information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    id: i32,
    name: String,
    email: String,
    description: String,
    servers: Servers,
}

impl UserInfo {
    /// Take all value to return `UserInfo`
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
