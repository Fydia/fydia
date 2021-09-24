use crate::{instance::Instance, server::Servers};
use fydia_utils::hash;
use gotham::hyper::HeaderMap;
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
    pub password: String,
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
            password: hash(password.into()),
            server: Servers::new(),
            email: email.into(),
        }
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
    /*pub async fn get_user(&self, executor: &FydiaPool) -> Option<User> {
        User::get_user_by_id(self.id, executor).await
    }*/
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
