//! This module is related to server

use crate::channel::{ChannelError, ChannelId};
use crate::emoji::Emoji;
use crate::roles::Role;
use crate::sqlerror::GenericSqlError;
use crate::user::UserError;
use crate::utils::IdError;
use crate::{channel::Channel, user::UserId};
use fydia_utils::generate_string;
use fydia_utils::{
    serde::{Deserialize, Serialize},
    serde_json,
};
use thiserror::Error;

/// `Server` contains all value of server
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct Server {
    pub id: ServerId,
    pub name: String,
    pub owner: UserId,
    pub icon: String,
    pub emoji: Vec<Emoji>,
    pub members: Members,
    pub roles: Vec<Role>,
    pub channel: Channels,
}

impl Server {
    /// Take a `Into<String>` value and `UserId` and return `Result<Server, String>`
    ///
    /// # Errors
    /// Return an error if name is empty or owner id is negative
    pub fn new<T: Into<String>>(name: T, owner: UserId) -> Result<Self, ServerError> {
        let name = name.into();

        if name.is_empty() {
            return Err(ServerError::EmptyNameServer);
        }

        if owner.0.is_not_set() {
            return Err(ServerError::UserIdUnvalid);
        }

        Ok(Self {
            name,
            owner,
            ..Default::default()
        })
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: ServerId::new(generate_string(30)),
            name: String::new(),
            owner: UserId::default(),
            icon: String::new(),
            emoji: Vec::new(),
            members: Members::default(),
            roles: Vec::new(),
            channel: Channels::new(),
        }
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `ServerError` represents all errors of `Server`
pub enum ServerError {
    #[error("Server have an empty name")]
    EmptyNameServer,
    #[error("Owner have an unvalid id")]
    UserIdUnvalid,
    #[error("Cannot get the server with this id")]
    CannotGetById,
    #[error("Cannot Convert the model to struct")]
    ModelToStruct,
    #[error("Cannot get owner of the server")]
    CannotGetOwner,
    #[error("This server is already join")]
    AlreadyJoin,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
    #[error("{0}")]
    CannotGetMembers(MembersError),
    #[error("{0}")]
    CannotGetChannel(ChannelError),
}

impl From<ChannelError> for ServerError {
    fn from(value: ChannelError) -> Self {
        Self::CannotGetChannel(value)
    }
}

impl From<UserError> for ServerError {
    fn from(_: UserError) -> Self {
        Self::CannotGetOwner
    }
}

impl From<MembersError> for ServerError {
    fn from(value: MembersError) -> Self {
        Self::CannotGetMembers(value)
    }
}

impl From<IdError> for ServerError {
    fn from(value: IdError) -> Self {
        match value {
            IdError::IdUnset => Self::UserIdUnvalid,
        }
    }
}

impl From<GenericSqlError> for ServerError {
    fn from(value: GenericSqlError) -> Self {
        Self::GenericSqlError(Box::new(value))
    }
}

/// `ServerId` contains a String that represent an id of a server
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
#[serde(crate = "fydia_utils::serde")]
pub struct ServerId {
    pub id: String,
}

impl ServerId {
    /// Take a id as `Into<String>` and return `ServerId`
    pub fn new<T: Into<String>>(id: T) -> Self {
        Self { id: id.into() }
    }

    /// Take a `Into<String>` value and compare with `Server.id`
    pub fn eq<T: Into<String>>(&mut self, id: T) -> bool {
        self.id == id.into()
    }
}

/// `Servers` contains all server of an `User`
#[derive(Default, Hash, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Eq)]
#[serde(crate = "fydia_utils::serde")]
pub struct Servers(pub Vec<ServerId>);

impl Servers {
    /// Take a `ServerId` and check if is already in `Vec<ServerId>`
    pub fn is_join(&self, server_id: &ServerId) -> bool {
        for i in self.0.iter() {
            if cfg!(debug_assertion) {
                let serverid_id = &i.id;
                let cmp_serverid_id = &server_id.id;
                println!("`{serverid_id}`/`{cmp_serverid_id}`");
            }

            if i.id == server_id.id {
                return true;
            }
        }

        false
    }

    /// Take a `Into<String>` and return `ServerId` if exists in `Vec<ServerId>`
    pub fn get<T: Into<String>>(&self, server_id: T) -> Option<ServerId> {
        let server_id = server_id.into();
        for i in self.0.iter() {
            if i.id == server_id {
                return Some(i.clone());
            }
        }

        None
    }
    /// Return a default `Servers` with an empty `Vec`
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

/// `Members` contains number of member and all `User` in a `Server`
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(crate = "fydia_utils::serde")]
pub struct Members {
    pub members: Vec<UserId>,
}

impl Members {
    /// Return an `Members` with value given
    pub fn new(members: Vec<UserId>) -> Self {
        Self { members }
    }

    /// Serialize `Members` as Json
    ///
    /// # Errors
    /// Return an error if `Members` isn't serializable
    pub fn to_string(&self) -> Result<String, MembersError> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(err) => Err(MembersError::CannotSerialize(err)),
        }
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `MembersError` represents all errors of `Members`
pub enum MembersError {
    #[error("{0}")]
    CannotSerialize(serde_json::Error),
    #[error("Cannot get users")]
    CannotGetUsers,
    #[error("Cannot get users with this expression")]
    CannotGetMembersWithThisExpr,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
}

impl From<IdError> for MembersError {
    fn from(_: IdError) -> Self {
        MembersError::CannotGetUsers
    }
}

impl From<UserError> for MembersError {
    fn from(_: UserError) -> Self {
        MembersError::CannotGetUsers
    }
}

impl From<GenericSqlError> for MembersError {
    fn from(value: GenericSqlError) -> Self {
        MembersError::GenericSqlError(Box::new(value))
    }
}

/// `Channels` contains all channel of a `Server`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "fydia_utils::serde")]
pub struct Channels(pub Vec<Channel>);

impl Channels {
    /// Return an empty `Channels`
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Check if a `ChannelId` is already exists
    pub fn is_exists(&self, channel_id: &ChannelId) -> bool {
        for i in &self.0 {
            if &i.id == channel_id {
                return true;
            }
        }

        false
    }

    /// Return Channel with same `ChannelId` given or None if nothing match
    pub fn get_channel(&self, channel_id: &ChannelId) -> Option<Channel> {
        for i in &self.0 {
            if &i.id == channel_id {
                return Some(i.clone());
            }
        }
        None
    }
}

impl Default for Channels {
    fn default() -> Self {
        Self::new()
    }
}
