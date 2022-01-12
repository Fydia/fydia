use std::fmt::Display;

use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

use crate::{
    server::ServerId,
    user::{User, UserId},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    Voice,
    Text,
    DirectMessage,
}

impl ChannelType {
    pub fn is_voice(&self) -> bool {
        self == &ChannelType::Voice
    }
    pub fn is_text(&self) -> bool {
        self == &ChannelType::Text
    }
}

impl Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelType::Voice => write!(f, "VOICE"),
            ChannelType::Text => write!(f, "TEXT"),
            ChannelType::DirectMessage => write!(f, "DIRECT_MESSAGE"),
        }
    }
}

impl ChannelType {
    pub fn from_string(toparse: String) -> Self {
        match toparse.to_uppercase().as_str() {
            "VOICE" => Self::Voice,
            "DIRECT_MESSAGE" => Self::DirectMessage,
            _ => Self::Text,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectMessage {
    pub users: DirectMessageValue,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum DirectMessageValue {
    Users(Vec<User>),
    UsersId(Vec<UserId>),
}

impl DirectMessage {
    pub fn new(users: Vec<UserId>) -> Self {
        Self {
            users: DirectMessageValue::UsersId(users),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParentId {
    #[serde(rename = "direct_message")]
    DirectMessage(DirectMessage),
    #[serde(rename = "server_id")]
    ServerId(ServerId),
}

impl ParentId {
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ChannelId {
    pub id: String,
}

impl ChannelId {
    pub fn new<T: Into<String>>(id: T) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: ChannelId,
    #[serde(flatten)]
    pub parent_id: ParentId,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
}

impl Channel {
    pub fn new<T: Into<String>>(name: T, description: T, channel_type: ChannelType) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            channel_type,
            ..Default::default()
        }
    }

    pub fn new_with_parentid<T: Into<String>>(
        name: T,
        description: T,
        parent_id: ParentId,
        channel_type: ChannelType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            channel_type,
            parent_id,
            ..Default::default()
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            id: ChannelId::new(generate_string(15)),
            parent_id: ParentId::ServerId(ServerId::new(String::new())),
            name: String::new(),
            description: String::new(),
            channel_type: ChannelType::Text,
        }
    }
}
