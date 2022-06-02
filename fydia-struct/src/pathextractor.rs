//! This module is related to extractor of path

use fydia_utils::serde::Deserialize;

/// Path Extractor for path with /api/server/{}/channel/{}
#[allow(missing_docs)]
#[derive(Deserialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct ChannelExtractor {
    pub serverid: String,
    pub channelid: String,
}

/// Path Extractor for path with /api/server/{}
#[allow(missing_docs)]
#[derive(Deserialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct ServerExtractor {
    pub serverid: String,
}

/// Path Extractor for path with
#[allow(missing_docs)]
#[derive(Deserialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct UserExtractor {
    pub id: String,
}
/// Path Extractor for path with
#[allow(missing_docs)]
#[derive(Deserialize, Debug)]
#[serde(crate = "fydia_utils::serde")]
pub struct DmExtractor {
    pub id: String,
}
