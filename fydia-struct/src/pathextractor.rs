#[derive(serde::Deserialize, Debug)]
pub struct ChannelExtractor {
    pub serverid: String,
    pub channelid: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ServerExtractor {
    pub serverid: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct UserExtractor {
    pub id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DmExtractor {
    pub id: String,
}
