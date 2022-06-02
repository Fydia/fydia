//! This module is related to event

use crate::channel::ChannelId;
use crate::server::ServerId;
use crate::{messages::Message, user::UserId};
use fydia_utils::serde::{Deserialize, Serialize};

/// `Event` represent the message by websocket.
///
///# Examples
///```
///use fydia_struct::event::EventContent;
///use fydia_struct::server::ServerId;
///use fydia_struct::event::Event;
///
///let event = Event::new(ServerId::new(String::new()), EventContent::MessageDelete(String::new()));
///```
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "fydia_utils::serde")]
pub struct Event {
    pub server_id: ServerId,
    pub content: EventContent,
}

impl Event {
    /// Take `server_id` as `ServerId` and content as `EventContent`
    /// and return `Event`
    ///
    ///# Examples
    ///```
    ///use fydia_struct::event::EventContent;
    ///use fydia_struct::server::ServerId;
    ///use fydia_struct::event::Event;
    ///
    ///let event = Event::new(ServerId::new(String::new()), EventContent::MessageDelete { message_id: String::new() });
    ///```
    pub fn new(server_id: ServerId, content: EventContent) -> Self {
        Self { server_id, content }
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(crate = "fydia_utils::serde")]
#[serde(tag = "type")]
pub enum EventContent {
    Message {
        content: Box<Message>,
    },
    MessageDelete {
        message_id: String,
    },
    MessageUpdate {
        message_id: String,
        update: Box<Message>,
    },
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
