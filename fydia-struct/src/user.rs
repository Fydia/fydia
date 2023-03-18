//! This module is related to user

use crate::{
    instance::Instance,
    server::{ServerId, Servers},
    sqlerror::GenericSqlError,
    utils::{Id, IdError},
};
use fydia_crypto::password::hash;
use fydia_utils::http::HeaderMap;

use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json,
};
use thiserror::Error;

/// `User` contains all value of user
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Eq, Default)]
#[serde(crate = "fydia_utils::serde")]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub instance: Instance,
    #[serde(skip)]
    pub token: Token,
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
    /// Use it with precausion
    pub fn insert_server(&mut self, server_short_id: &ServerId) {
        self.servers.0.push(server_short_id.clone());
    }

    /// Return JSON Value for get user info
    ///
    /// # Errors
    /// Return an error if:
    /// * `Id` is unset
    pub fn self_json_output(&self) -> Result<impl Serialize, String> {
        #[derive(Serialize)]
        #[serde(crate = "fydia_utils::serde")]
        struct JsonBuf {
            id: u32,
            name: String,
            email: String,
            description: String,
        }

        Ok(JsonBuf {
            id: self.id.0.clone().get_id().map_err(|err| err.to_string())?,
            name: self.name.clone(),
            email: self.email.clone(),
            description: self.description.clone().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `UserError` represents all errors of `Users`
pub enum UserError {
    #[error("No user with this email")]
    CannotGetByEmail,
    #[error("Password don't match")]
    PasswordError,
    #[error("Password is empty")]
    EmptyPassword,
    #[error("No user with this id")]
    CannotGetById,
    #[error("No user with this token")]
    CannotGetByToken,
    #[error("Cannot update the token")]
    CannotUpdateToken,
    #[error("Cannot update the name")]
    CannotUpdateName,
    #[error("Cannot update the password")]
    CannotUpdatePassword,

    #[error("Cannot get roles of user")]
    CannotGetRolesOfUser,
    #[error("Cannot convert database model to struct")]
    ModelToStruct,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
    #[error("{0}")]
    Other(String),
}

impl From<TokenError> for UserError {
    fn from(_: TokenError) -> Self {
        UserError::CannotGetByToken
    }
}

impl From<IdError> for UserError {
    fn from(_: IdError) -> Self {
        UserError::CannotGetById
    }
}

impl From<String> for UserError {
    fn from(value: String) -> Self {
        UserError::Other(value)
    }
}

impl From<GenericSqlError> for UserError {
    fn from(value: GenericSqlError) -> Self {
        Self::GenericSqlError(Box::new(value))
    }
}

/// `UserId` contains id of `User`
#[allow(missing_docs)]
#[derive(Debug, Hash, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(crate = "fydia_utils::serde")]
pub struct UserId(pub Id<u32>);

impl Default for UserId {
    fn default() -> Self {
        Self(Id::Unset)
    }
}

impl UserId {
    /// Return a new `UserId`
    pub fn new(id: u32) -> Self {
        Self(Id::Id(id))
    }

    /// Serialize `UserId` as Json
    ///
    /// # Errors
    /// Return an error if `UserId` cannot be convert as Json
    pub fn to_string(&self) -> Result<String, String> {
        serde_json::to_string(&self).map_err(|f| f.to_string())
    }
}

/// Header name for the user token
pub const HEADERNAME: &str = "Authorization";

/// `Token` contains the token of a `User`
#[derive(Debug, Default, Clone, PartialOrd, PartialEq, Eq)]
pub struct Token(Option<String>);

impl Token {
    /// Create a new token from a String
    pub fn new(token: String) -> Self {
        Self(Some(token))
    }

    /// Create an empty Token
    pub fn null() -> Self {
        Self(None)
    }

    /// Get if Token is empty
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }

    //// Get token
    ///
    /// # Errors
    /// Return error if Token is empty
    pub fn get_token(&self) -> Result<String, TokenError> {
        if let Some(token) = &self.0 {
            return Ok(token.clone());
        }

        Err(TokenError::NoToken)
    }

    /// Return a Token from HTTP headers
    pub fn from_headervalue(headers: &HeaderMap) -> Token {
        if let Some(token) = headers.get(HEADERNAME) {
            if let Ok(token) = token.to_str() {
                return Token::new(token.to_string());
            }
        }

        Token::null()
    }
}

#[derive(Debug, Error)]
/// `TokenError` is error enum of `Token`
pub enum TokenError {
    /// `Token` is empty
    #[error("Token is empty")]
    NoToken,
}
