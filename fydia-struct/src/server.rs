//! This module is related to server

use crate::channel::ChannelId;
use crate::emoji::Emoji;
use crate::roles::Role;
use crate::{channel::Channel, user::UserId};
use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

/// `Server` contains all value of server
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub fn new<T: Into<String>>(name: T, owner: UserId) -> Result<Self, String> {
        let name = name.into();

        if name.is_empty() {
            return Err(String::from("Name server is empty"));
        }

        if owner.0.is_negative() {
            return Err(String::from("UserId is negative"));
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
            owner: UserId::new(-1),
            icon: String::new(),
            emoji: Vec::new(),
            members: Members {
                count: 0,
                members: Vec::new(),
            },
            roles: Vec::new(),
            channel: Channels::new(),
        }
    }
}

/// `ServerId` contains a String that represent an id of a server
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
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
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Members {
    count: i32,
    pub members: Vec<UserId>,
}

impl Members {
    /// Return an empty  `Members`
    pub fn new() -> Self {
        Self {
            count: 0,
            members: Vec::new(),
        }
    }
    /// Return a new `Members` with value given
    pub fn new_with(count: i32, members: Vec<UserId>) -> Self {
        Self { count, members }
    }

    /// Add a new `User` in members
    pub fn push(&mut self, user: UserId) {
        self.count += 1;
        self.members.push(user);
    }

    /// Remove a `User` in members
    ///
    /// # Errors
    /// Return an error if user doesn't exist in array
    pub fn remove(&mut self, user: &UserId) -> Result<(), String> {
        for (n, i) in (&self.members).iter().enumerate() {
            if i.0 == user.0 {
                self.members.remove(n);
                self.count -= 1;
                return Ok(());
            }
        }

        Err("Not Found".to_string())
    }

    /// Serialize `Members` as Json
    ///
    /// # Errors
    /// Return an error if `Members` isn't serializable
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for Members {
    fn default() -> Self {
        Self::new()
    }
}

/// `Channels` contains all channel of a `Server`
#[derive(Clone, Debug, Serialize, Deserialize)]
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
