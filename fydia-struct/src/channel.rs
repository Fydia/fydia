use std::fmt::Display;

use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

use crate::server::ServerId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    Voice,
    Text,
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
        }
    }
}

impl ChannelType {
    pub fn from_string(toparse: String) -> Self {
        match toparse.to_uppercase().as_str() {
            "VOICE" => Self::Voice,
            _ => Self::Text,
        }
    }
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
    pub server_id: ServerId,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
}

impl Channel {
    pub fn new() -> Self {
        let gen = generate_string(15);
        Self {
            id: gen,
            server_id: ServerId::new(String::new()),
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
