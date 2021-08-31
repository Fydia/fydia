use crate::channel::ChannelId;
use crate::user::User;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Timelike, Utc};
use fydia_utils::generate_string;
use serde::de::{Error, Unexpected, Visitor};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageType {
    TEXT,
    FILE,
    VIDEO,
    PHOTO,
    URL,
    AUDIO,
}

impl MessageType {
    pub fn to_string(&self) -> String {
        match self {
            MessageType::TEXT => String::from("TEXT"),
            MessageType::FILE => String::from("FILE"),
            MessageType::VIDEO => String::from("VIDEO"),
            MessageType::PHOTO => String::from("PHOTO"),
            MessageType::AUDIO => String::from("AUDIO"),
            MessageType::URL => String::from("URL"),
        }
    }

    pub fn from_string(from: String) -> Option<Self> {
        match from.to_uppercase().as_str() {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub message_type: MessageType,
    pub edited: bool,
    pub timestamp: SqlDate,
    #[serde(rename = "channel")]
    pub channel_id: ChannelId,
    #[serde(rename = "author")]
    pub author_id: User,
}

impl Message {
    pub fn new(
        content: String,
        message_type: MessageType,
        edited: bool,
        timestamp: SqlDate,
        author_id: User,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            id: generate_string(32),
            content,
            message_type,
            edited,
            timestamp,
            author_id,
            channel_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SqlDate(pub DateTime<Utc>);

impl SqlDate {
    pub fn new(date: DateTime<Utc>) -> Self {
        Self { 0: date }
    }
    pub fn parse_string(parse: String) -> Self {
        let datetime = DateTime::parse_from_str(parse.as_str(), "%F %T").unwrap();
        Self::new_fixed(datetime)
    }
    pub fn now() -> Self {
        Self {
            0: DateTime::from(SystemTime::now()),
        }
    }

    pub fn new_fixed(date: DateTime<FixedOffset>) -> Self {
        Self {
            0: DateTime::from(date),
        }
    }
}

impl Serialize for SqlDate {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        //serde::ser::SerializeStruct::serialize_field(&mut serializer, "timestamp")
        serializer
            .serialize_str(format!("{}", datetime_to_sqltime(DateTime::from(self.0))).as_str())
    }
}

impl<'de> Deserialize<'de> for SqlDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(string_deser) = String::deserialize(deserializer) {
            if let Ok(e) = NaiveDateTime::parse_from_str(string_deser.as_str(), "%Y-%m-%d %H:%M:%S")
            {
                Ok(SqlDate::new(DateTime::<Utc>::from_utc(e, Utc)))
            } else {
                Err(de::Error::custom("Error on SqlDate"))
            }
        } else {
            Err(de::Error::custom("Error on SqlDate"))
        }
    }
}

impl<'de> Visitor<'de> for SqlDate {
    type Value = SqlDate;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("An string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Ok(e) = DateTime::parse_from_str(v, "F T") {
            let value = SqlDate::new(DateTime::from(e));
            Ok(value)
        } else {
            Err(de::Error::invalid_type(Unexpected::Str("Error"), &self))
        }
    }
}

pub fn datetime_to_sqltime(date: DateTime<Utc>) -> String {
    // F T
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
