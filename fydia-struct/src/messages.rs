//! This module related to message

use crate::channel::ChannelId;
use crate::user::User;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Timelike, Utc};
use fydia_utils::generate_string;
use serde::de::{Error, Unexpected, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
/// `MessageType` reprensent the type of Message.
/// This enum is used in the Message Json$
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    /// Parse a str to convert it in MessageType
    ///
    /// If str cannot be convert None is return
    ///
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
    pub fn from_string<T: Into<String>>(from: T) -> Option<Self> {
        match from.into().to_uppercase().as_str() {
            "TEXT" => Some(Self::TEXT),
            "FILE" => Some(Self::FILE),
            "URL" => Some(Self::URL),
            "VIDEO" => Some(Self::VIDEO),
            "PHOTO" => Some(Self::PHOTO),
            "AUDIO" => Some(Self::AUDIO),
            _ => None,
        }
    }
}

/// Message contains all value of a message.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    ) -> Result<Self, String> {
        let content = content.into();

        if content.is_empty() {
            return Err(String::from("Content is empty"));
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

/// Date contains a `DateTime<Utc>`
#[derive(Debug, Clone)]
pub struct Date(pub DateTime<Utc>);

impl Date {
    /// Create a new `Date` with a specific `DateTime<Utc>`
    ///
    /// Prefer use Date::now() for current Date
    pub fn new(date: DateTime<Utc>) -> Self {
        Self { 0: date }
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

    /// Create a new `Date` with current time.
    pub fn now() -> Self {
        Self {
            0: DateTime::from(SystemTime::now()),
        }
    }

    /// Create a new `Date` with a new `DateTime<FixedOffset>`
    pub fn new_fixed(date: DateTime<FixedOffset>) -> Self {
        Self {
            0: DateTime::from(date),
        }
    }
    
    /// Create a new `Date` with minimal DateTime 
    pub fn null() -> Self {
        Self(DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc))
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
        if let Ok(string_deser) = String::deserialize(deserializer) {
            if let Ok(e) = NaiveDateTime::parse_from_str(string_deser.as_str(), "%Y-%m-%d %H:%M:%S")
            {
                Ok(Date::new(DateTime::<Utc>::from_utc(e, Utc)))
            } else {
                Err(de::Error::custom("Error on Date"))
            }
        } else {
            Err(de::Error::custom("Error on Date"))
        }
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

/// Format `DateTime<Utc>` as "%F %T" and return String
pub fn datetime_to_sqltime(date: DateTime<Utc>) -> String {
    format!(
        "{:0>#4}-{:0>#2}-{:0>#2} {:0>#2}:{:0>#2}:{:0>#2}",
        date.date().year(),
        date.date().month(),
        date.date().day(),
        date.time().hour(),
        date.time().minute(),
        date.time().second()
    )
}
