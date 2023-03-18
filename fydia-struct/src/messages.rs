//! This module related to message

use crate::channel::ChannelId;
use crate::sqlerror::GenericSqlError;
use crate::user::User;
use crate::utils::IdError;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use fydia_utils::generate_string;
use fydia_utils::serde::{
    de::{self, Error, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use thiserror::Error;
/// `MessageType` reprensent the type of Message.
/// This enum is used in the Message Json$
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(crate = "fydia_utils::serde")]
pub enum MessageType {
    TEXT,
    FILE,
    VIDEO,
    PHOTO,
    URL,
    AUDIO,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::TEXT => write!(f, "TEXT"),
            MessageType::FILE => write!(f, "FILE"),
            MessageType::VIDEO => write!(f, "VIDEO"),
            MessageType::PHOTO => write!(f, "PHOTO"),
            MessageType::AUDIO => write!(f, "AUDIO"),
            MessageType::URL => write!(f, "URL"),
        }
    }
}

impl MessageType {
    /// Parse a str to convert it in `MessageType`
    ///
    /// # Errors
    /// Return an error if type is unknow
    /// # Examples
    ///
    /// ```
    /// use fydia_struct::messages::MessageType;
    ///
    /// assert_eq!(Some(MessageType::TEXT), MessageType::from_string("text"));
    /// ```
    ///
    ///```
    ///use fydia_struct::messages::MessageType;
    ///
    ///assert_eq!(None, MessageType::from_string("NOVALUE"));
    ///```
    pub fn from_string<T: Into<String>>(from: T) -> Result<Self, MessageTypeError> {
        let from = from.into();
        match from.to_uppercase().as_str() {
            "TEXT" => Ok(Self::TEXT),
            "FILE" => Ok(Self::FILE),
            "URL" => Ok(Self::URL),
            "VIDEO" => Ok(Self::VIDEO),
            "PHOTO" => Ok(Self::PHOTO),
            "AUDIO" => Ok(Self::AUDIO),
            _ => Err(MessageTypeError::UnknowType(from)),
        }
    }
}

impl Default for MessageType {
    fn default() -> Self {
        Self::TEXT
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `MessageTypeError` represents all errors of `MessageType`
pub enum MessageTypeError {
    #[error("Unknow type ; `{0}`")]
    UnknowType(String),
}

/// Message contains all value of a message.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(crate = "fydia_utils::serde")]
pub struct Message {
    pub id: String,
    pub content: String,
    pub message_type: MessageType,
    pub edited: bool,
    pub timestamp: Date,
    #[serde(rename = "channel")]
    pub channel_id: ChannelId,
    #[serde(rename = "author")]
    pub author_id: User,
}

impl Message {
    /// Create a new `Message` with all needed values.
    ///
    /// # Errors
    /// Return an error if :
    /// * content is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use fydia_struct::{messages::{Message, MessageType, Date}, user::User, channel::ChannelId};
    ///
    /// Message::new("EMPTY", MessageType::TEXT, false, Date::now(), User::default(),
    /// ChannelId::new(String::from("THISISANIDOF15C")));
    /// ```
    pub fn new<T: Into<String>>(
        content: T,
        message_type: MessageType,
        edited: bool,
        timestamp: Date,
        author_id: User,
        channel_id: ChannelId,
    ) -> Result<Self, MessageError> {
        let content = content.into();

        if content.is_empty() {
            return Err(MessageError::ContentEmpty);
        }

        Ok(Self {
            id: generate_string(32),
            content,
            message_type,
            edited,
            timestamp,
            author_id,
            channel_id,
        })
    }
}

#[derive(Debug, Error)]
#[allow(missing_docs)]
/// `MessageError` represents all errors of `Message`
pub enum MessageError {
    #[error("Message's content cannot be empty")]
    ContentEmpty,
    #[error("Cannot convert Message in ActiveModel")]
    CannotIntoActiveModel,
    #[error("No message with this id")]
    CannotGetById,
    #[error("Cannot convert the model to the struct")]
    ModelToStruct,
    #[error("{0}")]
    GenericSqlError(Box<GenericSqlError>),
}

impl From<IdError> for MessageError {
    fn from(value: IdError) -> Self {
        match value {
            IdError::IdUnset => Self::CannotIntoActiveModel,
        }
    }
}

impl From<GenericSqlError> for MessageError {
    fn from(value: GenericSqlError) -> Self {
        Self::GenericSqlError(Box::new(value))
    }
}

/// Date contains a `DateTime<Utc>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date(pub DateTime<Utc>);

impl Date {
    /// Create a new `Date` with a specific `DateTime<Utc>`
    ///
    /// Prefer use `Date::now()` for current Date
    pub fn new(date: DateTime<Utc>) -> Self {
        Self(date)
    }
    /// Create a new `Date` with `NaiveDateTime`
    pub fn parse_from_naivetime(naivetime: NaiveDateTime) -> Self {
        Self(DateTime::from_utc(naivetime, Utc))
    }

    /// Create a new `Date` from a String
    ///
    /// ```
    /// use fydia_struct::messages::Date;
    ///
    /// let date = Date::parse_string("2020-01-01 00:00:00");
    /// ```
    pub fn parse_string<T: Into<String>>(parse: T) -> Option<Self> {
        if let Ok(datetime) = DateTime::parse_from_str(parse.into().as_str(), "%F %T") {
            Some(Self::new_fixed(datetime))
        } else {
            None
        }
    }

    /// Create a new `Date` from a Timestamp
    ///
    /// ```
    /// use fydia_struct::messages::Date;
    ///
    /// let date = Date::parse_timestamp(1647285703);
    /// ```
    pub fn parse_timestamp(parse: i64) -> Option<Self> {
        NaiveDateTime::from_timestamp_opt(parse, 0)
            .map(|datetime| Self(DateTime::from_utc(datetime, Utc)))
    }

    /// Create a new `Date` with current time.
    pub fn now() -> Self {
        Self(DateTime::from(SystemTime::now()))
    }

    /// Create a new `Date` with a new `DateTime<FixedOffset>`
    pub fn new_fixed(date: DateTime<FixedOffset>) -> Self {
        Self(DateTime::from(date))
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::now()
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0.timestamp())
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(timestamp) = i32::deserialize(deserializer) {
            return Date::parse_timestamp(i64::from(timestamp))
                .ok_or_else(|| de::Error::custom("Error on Date"));
        }

        Err(de::Error::custom("Error on Date"))
    }
}

impl<'de> Visitor<'de> for Date {
    type Value = Date;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("An string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(e) = DateTime::parse_from_str(v, "F T") {
            let value = Date::new(DateTime::from(e));
            Ok(value)
        } else {
            Err(de::Error::invalid_type(Unexpected::Str("Error"), &self))
        }
    }
}
