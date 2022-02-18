//! All structs for channels

use std::fmt::Display;

use fydia_utils::generate_string;
use serde::{Deserialize, Serialize};

use crate::{
    server::ServerId,
    user::{User, UserId},
};
/// ChannelType reprensent which type of channel is.
/// Voice, Text or DirectMessage
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChannelType {
    Voice,
    Text,
    DirectMessage,
}

impl ChannelType {
    /// Check if channel is voice
    pub fn is_voice(&self) -> bool {
        self == &ChannelType::Voice
    }
    /// Check if channel is text
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
    /// Take a `String` to return a `ChannelType`
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::channel::ChannelType;
    ///
    /// let channel_type = ChannelType::from_string("voice");
    /// assert_eq!(channel_type, ChannelType::Voice)
    /// ```
    pub fn from_string<T: Into<String>>(toparse: T) -> Self {
        match toparse.into().to_uppercase().as_str() {
            "VOICE" => Self::Voice,
            "DIRECT_MESSAGE" => Self::DirectMessage,
            _ => Self::Text,
        }
    }
}

/// DirectMessage reprensent a DM.
///
/// Inner of this struct is id of user or user.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectMessage {
    /// Users of direct message
    pub users: DirectMessageInner,
}

/// This is the inner of [`DirectMessage`]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum DirectMessageInner {
    /// List of [User](`fydia-struct::user::User`)
    Users(Vec<User>),
    /// List of [UserId](`fydia-struct::user::UserId`)
    UsersId(Vec<UserId>),
}

impl DirectMessage {
    /// Take a `Vec<UserId>` to return a DirectMessage
    ///
    /// # Examples
    /// 
    /// ```
    /// use fydia_struct::user::UserId;
    /// use fydia_struct::channel::DirectMessage;
    /// use fydia_struct::channel::DirectMessageInner;
    ///
    /// let direct_message = DirectMessage::new(vec![UserId::default(), UserId::default()]);
    ///
    /// if let DirectMessageInner::UsersId(inner) = direct_message.users {
    ///     assert_eq!(inner, vec![UserId::default(), UserId::default()]);
    /// }
    /// ```
    pub fn new(users: Vec<UserId>) -> Self {
        Self {
            users: DirectMessageInner::UsersId(users),
        }
    }
}

/// `ParentId` represents the parent of `Channel`
///
/// DirectMessage contains `DirectMessage` with `Vec<User>` or `Vec<UserId>`
///
/// ServerId contains `ServerId`
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParentId {
    #[serde(rename = "direct_message")]
    DirectMessage(DirectMessage),
    #[serde(rename = "server_id")]
    ServerId(ServerId),
}

impl ParentId {
    /// Serialize `ParentId` in Json and return a `Result<String, String>`
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::user::UserId;
    /// use fydia_struct::channel::ParentId;
    /// use fydia_struct::channel::DirectMessage;
    ///
    ///let parent_id = ParentId::DirectMessage(DirectMessage::new(vec![
    ///    UserId::default(),
    ///    UserId::default(),
    ///])).to_string().unwrap();
    ///
    /// assert_eq!(parent_id, r#"{"direct_message":{"users":[{"id":-1},{"id":-1}]}}"#)
    /// ```
    pub fn to_string(&self) -> Result<String, String> {
        match serde_json::to_string(self) {
            Ok(string) => Ok(string),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// ChannelId is Id of a Channel
///
/// Can be used to get a Channel and pass a channel in function without all data.
///
/// # Examples
/// ```
/// use fydia_struct::channel::ChannelId;
///
/// let channel_id = ChannelId::new("THISISACHANNELIDWITHMORETHAN15ASLENGHT");
/// assert_eq!(channel_id.id, String::from("THISISACHANNELI"))
/// ```
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ChannelId {
    /// Id of Channel
    pub id: String,
}

impl ChannelId {
    /// Take a `T` value that impl `Into<String>` and return `ChannelId`
    ///
    /// If the length of `T` is more than 15, `T` will be split.
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::channel::ChannelId;
    ///
    /// let channel_id = ChannelId::new("THISISACHANNELIDWITHMORETHAN15ASLENGHT");
    /// assert_eq!(channel_id.id, String::from("THISISACHANNELI"))
    /// ```
    pub fn new<T: Into<String>>(id: T) -> Self {
        Self {
            id: id.into().split_at(15).0.to_string(),
        }
    }
}

/// `Channel` is the struct that contains all information of a channel
///
/// `Channel.id` is generate randomly.
///
/// Return `Err(String)` when `name` is empty
///# Examples
///```
/// use fydia_struct::channel::{Channel, ChannelType};
///
/// let channel = Channel::new("name", "desc", ChannelType::Text);
///```
/// ## Error
/// ```should_panic
///  use fydia_struct::channel::ChannelType;
///  use fydia_struct::channel::Channel;
///
///  // This will be panic because there is no name
///  let channel = Channel::new("", "desc", ChannelType::Text).unwrap();
/// ```
///
#[allow(missing_docs)]
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
    /// Take name, description as a `T` value that impl `Into<String>`
    /// and channel_type as `ChannelType` to return `Channel`
    ///
    /// `Channel.id` is generate randomly.
    ///
    /// Return `Err(String)` when `name` is empty
    ///
    ///# Examples
    ///```
    /// use fydia_struct::channel::{Channel, ChannelType};
    ///
    /// let channel = Channel::new("name", "desc", ChannelType::Text);
    ///```
    /// ## Error
    /// ```should_panic
    /// use fydia_struct::channel::{Channel, ChannelType};
    ///  // This will be panic because there is no name
    ///  let channel = Channel::new("", "desc", ChannelType::Text).unwrap();
    /// ```
    pub fn new<T: Into<String>>(
        name: T,
        description: T,
        channel_type: ChannelType,
    ) -> Result<Self, String> {
        let name = name.into();
        let description = description.into();

        if name.is_empty() {
            return Err(String::from("Name is empty"));
        }

        Ok(Self {
            name,
            description,
            channel_type,
            ..Default::default()
        })
    }
    /// Take name, description as a `T` value that implements `Into<String>`
    /// and channel_type as `ChannelType` and parent_id as `ParentId`
    /// to return `Channel`
    ///
    /// `Channel.id` is generate randomly.
    ///
    /// Return `Err(String)` when `name` is empty
    ///
    ///# Examples
    ///```
    /// use fydia_struct::channel::{Channel, ChannelType, ParentId};
    /// use fydia_struct::server::ServerId;
    ///
    /// let channel = Channel::new_with_parentid("name", "desc",ParentId::ServerId(ServerId::new(String::new())), ChannelType::Text);
    ///```
    /// ## Error
    /// ```should_panic
    ///  use fydia_struct::channel::{Channel, ChannelType, ParentId};
    ///  use fydia_struct::server::ServerId;
    ///
    ///  // This will be panic because there is no name
    ///  let channel = Channel::new_with_parentid("", "desc", ParentId::ServerId(ServerId::new(String::new())), ChannelType::Text).unwrap();
    /// ```
    ///
    pub fn new_with_parentid<T: Into<String>>(
        name: T,
        description: T,
        parent_id: ParentId,
        channel_type: ChannelType,
    ) -> Result<Self, String> {
        let mut channel = Self::new(name, description, channel_type)?;

        channel.parent_id = parent_id;

        Ok(channel)
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
