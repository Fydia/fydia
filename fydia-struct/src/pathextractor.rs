//! This module is related to extractor of path

/// Path Extractor for path with /api/server/{}/channel/{}
#[allow(missing_docs)]
#[derive(serde::Deserialize, Debug)]
pub struct ChannelExtractor {
    pub serverid: String,
    pub channelid: String,
}


/// Path Extractor for path with /api/server/{} 
#[allow(missing_docs)]
#[derive(serde::Deserialize, Debug)]
pub struct ServerExtractor {
    pub serverid: String,
}

/// Path Extractor for path with
#[allow(missing_docs)]
#[derive(serde::Deserialize, Debug)]
pub struct UserExtractor {
    pub id: String,
}
/// Path Extractor for path with
#[allow(missing_docs)] 
#[derive(serde::Deserialize, Debug)]
pub struct DmExtractor {
    pub id: String,
}
