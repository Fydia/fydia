//! All structs for channels

use crate::server::{MembersError, ServerError, ServerId};
use crate::sqlerror::{GenericError, GenericSqlError};
use fydia_utils::generate_string;
use fydia_utils::serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
/// `ChannelType` reprensent which type of channel is.
/// Voice, Text or `DirectMessage`
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "fydia_utils::serde")]
pub enum ChannelType {
    Voice = 0,
    Text = 1,
    DirectMessage = 2,
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

    /// Take a `u32` to return a `ChannelType`
    ///
    /// # Examples
    /// ```
    /// use fydia_struct::channel::ChannelType;
    ///
    /// let channel_type = ChannelType::from_int(0);
    /// assert_eq!(channel_type, ChannelType::Voice)
    /// ```
    pub fn from_int(toparse: u32) -> Self {
        match toparse {
            0 => Self::Voice,
            2 => Self::DirectMessage,
            _ => Self::Text,
        }
    }
}

/// `ChannelId` is Id of a Channel
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
#[serde(crate = "fydia_utils::serde")]
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
#[serde(crate = "fydia_utils::serde")]
pub struct Channel {
    pub id: ChannelId,
    #[serde(flatten)]
    pub parent_id: ServerId,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
}

impl Channel {
    /// Take name, description as a `T` value that impl `Into<String>`
    /// and `channel_type` as `ChannelType` to return `Channel`
    ///
    /// `Channel.id` is generate randomly.
    ///
    /// # Errors
    /// Return an error if:
    /// * name is empty
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
    ) -> Result<Self, ChannelError> {
        let name = name.into();
        let description = description.into();

        if name.is_empty() {
            return Err(ChannelError::EmptyName);
        }

        Ok(Self {
            name,
            description,
            channel_type,
            ..Default::default()
        })
    }
    /// Take name, description as a `T` value that implements `Into<String>`
    /// and `channel_type` as `ChannelType` and `parent_id` as `ServerId`
    /// to return `Channel`
    ///
    /// `Channel.id` is generate randomly.
    ///
    /// # Errors
    /// Return an error if :
    /// * name is empty
    ///
    ///# Examples
    ///```
    /// use fydia_struct::channel::{Channel, ChannelType};
    /// use fydia_struct::server::ServerId;
    ///
    /// let channel = Channel::new_with_serverid("name", "desc",ServerId::new(String::new()), ChannelType::Text);
    ///```
    /// ## Error
    /// ```should_panic
    ///  use fydia_struct::channel::{Channel, ChannelType};
    ///  use fydia_struct::server::ServerId;
    ///
    ///  // This will be panic because there is no name
    ///  let channel = Channel::new_with_serverid("", "desc", ServerId::new(String::new()), ChannelType::Text).unwrap();
    /// ```
    ///
    pub fn new_with_serverid<T: Into<String>>(
        name: T,
        description: T,
        parent_id: ServerId,
        channel_type: ChannelType,
    ) -> Result<Self, ChannelError> {
        let mut channel = Self::new(name, description, channel_type)?;

        channel.parent_id = parent_id;

        Ok(channel)
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            id: ChannelId::new(generate_string(15)),
            parent_id: ServerId::new(String::new()),
            name: String::new(),
            description: String::new(),
            channel_type: ChannelType::Text,
        }
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `ChannelError` represents all errors of `Channel`
pub enum ChannelError {
    #[error("Cannot get this channel from database")]
    CannotGetFromDatabase,
    #[error("Cannot update name in database")]
    CannotUpdateName,
    #[error("Cannot update description in database")]
    CannotUpdateDescription,
    #[error("Cannot the server")]
    CannotGetServer,
    #[error("Cannot the members")]
    CannotGetMembers,
    #[error("No channel with this id")]
    CannotGetById,
    #[error("Empty name")]
    EmptyName,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
}

impl From<ServerError> for ChannelError {
    fn from(_: ServerError) -> Self {
        Self::CannotGetServer
    }
}

impl From<MembersError> for ChannelError {
    fn from(_: MembersError) -> Self {
        Self::CannotGetMembers
    }
}

impl From<GenericSqlError> for ChannelError {
    fn from(value: GenericSqlError) -> Self {
        match value {
            GenericSqlError::CannotInsert(_) | GenericSqlError::CannotDelete(_) => {
                ChannelError::GenericSqlError(Box::new(value))
            }
            GenericSqlError::CannotUpdate(GenericError { error, set_column }) => {
                error!("{error}");
                if set_column.contains(&"description".to_string()) {
                    return Self::CannotUpdateDescription;
                }

                Self::CannotUpdateName
            }
        }
    }
}
