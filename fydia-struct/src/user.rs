//! This module is related to user

use crate::{
    instance::Instance,
    server::{ServerId, Servers},
    utils::Id,
};
use fydia_crypto::password::hash;
use fydia_utils::http::HeaderMap;
use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json,
};

/// `User` contains all value of user
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Eq, Default)]
#[serde(crate = "fydia_utils::serde")]
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
            id: self.id.0.clone().get_id()?,
            name: self.name.clone(),
            email: self.email.clone(),
            description: self.description.clone().unwrap_or_default(),
        })
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
#[derive(Debug)]
pub struct Token(pub String);

impl Token {
    /// Return a Token from HTTP headers
    pub fn from_headervalue(headers: &HeaderMap) -> Option<Token> {
        let token = headers.get(HEADERNAME)?;

        Some(Token(token.to_str().ok()?.to_string()))
    }
}
