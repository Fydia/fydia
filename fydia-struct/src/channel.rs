use std::fmt::Display;

use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

use crate::{server::ServerId, user::UserId};

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
    pub users: Vec<UserId>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParentId {
    #[serde(rename = "direct_messages")]
    DirectMessage(DirectMessage),
    #[serde(rename = "server_id")]
    ServerId(ServerId),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChannelId {
    pub id: String,
}

impl ChannelId {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub parent_id: ParentId,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
}

impl Channel {
    pub fn new() -> Self {
        let gen = generate_string(15);
        Self {
            id: gen,
            parent_id: ParentId::ServerId(ServerId::new(String::new())),
            name: String::new(),
            description: String::new(),
            channel_type: ChannelType::Text,
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}
