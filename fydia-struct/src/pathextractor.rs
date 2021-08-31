#[derive(serde::Deserialize, StateData, StaticResponseExtender, Debug)]
pub struct ChannelExtractor {
    pub serverid: String,
    pub channelid: String,
}

#[derive(serde::Deserialize, StateData, StaticResponseExtender, Debug)]
pub struct ServerExtractor {
    pub serverid: String,
}
