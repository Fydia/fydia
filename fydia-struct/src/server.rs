use crate::channel::Channel;
use crate::emoji::Emoji;
use crate::roles::Role;
use crate::user::User;
use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub shortid: String,
    pub name: String,
    pub owner: i32,
    pub icon: String,
    pub emoji: Vec<Emoji>,
    pub members: Members,
    pub roles: Vec<Role>,
    pub channel: Channels,
}

impl Server {
    pub fn new() -> Self {
        let gen = generate_string(30);
        Self {
            id: gen.clone(),
            shortid: gen.split_at(10).0.to_string(),
            name: String::new(),
            owner: 0,
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq)]
pub struct ServerId {
    pub id: String,
    #[serde(skip)]
    pub short_id: String,
}

impl ServerId {
    pub fn new(id: String) -> Self {
        let short_id = if !id.is_empty() {
            id.split_at(10).0.to_string()
        } else {
            id.clone()
        };

        Self { id, short_id }
    }

    pub fn eq(&mut self, id: String) -> bool {
        if self.short_id.is_empty() {
            let short_id = self.id.split_at(10).0.to_string();
            if self.short_id == short_id || self.id == id {
                return true;
            }
        }

        if self.short_id == id || self.id == id {
            return true;
        }

        false
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct Servers(pub Vec<ServerId>);

impl Servers {
    pub fn is_join(&self, server_id: ServerId) -> bool {
        for i in self.0.clone().iter_mut() {
            println!(
                "`{}`/`{}` => `{}`/`{}`",
                i.short_id, i.id, server_id.short_id, server_id.id
            );
            if i.short_id.is_empty() {
                i.short_id = i.id.split_at(10).0.to_string();
            }
            if i.short_id == server_id.short_id || i.id == server_id.id {
                return true;
            }
        }
        false
    }

    pub fn get(&self, server_id: String) -> Option<ServerId> {
        for i in self.0.clone().iter_mut() {
            if i.short_id.is_empty() {
                i.short_id = i.id.split_at(10).0.to_string();
            }

            if i.short_id == server_id || i.id == server_id {
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channels(pub Vec<Channel>);

impl Channels {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_exists(&self, channel_id: String) -> bool {
        for i in &self.0 {
            if i.id == channel_id {
                return true;
            }
        }
        false
    }

    pub fn get_channel(&self, channel_id: String) -> Option<Channel> {
        for i in &self.0 {
            if i.id == channel_id {
                return Some(i.clone());
            }
        }
        None
    }
}