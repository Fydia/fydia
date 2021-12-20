use crate::channel::ChannelId;
use crate::server::ServerId;
use crate::{messages::Message, user::UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub server: ServerId,
    pub content: EventContent,
}

impl Event {
    pub fn new(server: ServerId, content: EventContent) -> Self {
        Self { server, content }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventContent {
    Message {
        content: Message,
    },
    MessageDelete(String),
    MessageUpdate(String),
    UserChangeName(String),
    VocalChannelJoin(String),  // When user join a vocal channel
    VocalChannelLeave(String), // When user quit a vocal channel
    ServerJoin(String),        // When a user join
    ServerLeft(String),        // When a user quit
    ChannelCreate(String),
    ChannelUpdate(String),
    ChannelDelete(String),
    StartTyping {
        userid: UserId,
        channelid: ChannelId,
    },
    StopTyping {
        userid: UserId,
        channelid: ChannelId,
    },
}
