use crate::channel::ChannelId;
use crate::emoji::Emoji;
use crate::roles::Role;
use crate::user::User;
use crate::{channel::Channel, user::UserId};
use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn new<T: Into<String>>(name: T, owner: UserId) -> Result<Self, String> {
        let name = name.into();

        if name.is_empty() {
            return Err(String::from("Name server is empty"));
        }

        if owner.id.is_negative() {
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct ServerId {
    pub id: String,
}

impl ServerId {
    pub fn new<T: Into<String>>(id: T) -> Self {
        Self { id: id.into() }
    }

    pub fn eq<T: Into<String>>(&mut self, id: T) -> bool {
        self.id == id.into()
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct Servers(pub Vec<ServerId>);

impl Servers {
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

    pub fn get<T: Into<String>>(&self, server_id: T) -> Option<ServerId> {
        let server_id = server_id.into();
        for i in self.0.iter() {
            if i.id == server_id {
                return Some(i.clone());
            }
        }

        None
    }

    pub fn new() -> Self {
        Self(Vec::new())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Members {
    count: i32,
    pub members: Vec<User>,
}

impl Members {
    pub fn new() -> Self {
        Self {
            count: 0,
            members: Vec::new(),
        }
    }

    pub fn new_with(count: i32, members: Vec<User>) -> Self {
        Self { count, members }
    }

    pub fn push(&mut self, user: User) {
        self.count += 1;
        self.members.push(user);
    }

    pub fn remove(&mut self, user: User) -> Result<(), String> {
        for (n, i) in (&self.members).iter().enumerate() {
            if i.id == user.id {
                self.members.remove(n);
                self.count -= 1;
                return Ok(());
            }
        }

        Err("Not Found".to_string())
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Channels(pub Vec<Channel>);

impl Channels {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_exists(&self, channel_id: &ChannelId) -> bool {
        for i in &self.0 {
            if &i.id == channel_id {
                return true;
            }
        }

        false
    }

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
