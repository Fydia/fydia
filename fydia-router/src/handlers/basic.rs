use fydia_sql::{
    impls::{channel::SqlChannelId, server::SqlServerId, token::SqlToken},
    sqlpool::DbConnection,
};
use http::HeaderMap;

use fydia_struct::{
    channel::{Channel, ChannelId},
    server::{Server, ServerId},
    user::{Token, User},
};

#[derive(Debug)]
pub struct BasicValues;

impl BasicValues {
    pub async fn get_user(headers: &HeaderMap, executor: &DbConnection) -> Result<User, String> {
        let token = Token::from_headervalue(headers).ok_or("No token".to_string())?;
        token
            .get_user(executor)
            .await
            .ok_or("Wrong token".to_string())
    }

    pub async fn get_user_and_server_and_check_if_joined(
        headers: &HeaderMap,
        serverid: String,
        executor: &DbConnection,
    ) -> Result<(User, Server), String> {
        let user = Self::get_user(headers, executor).await?;
        let serverid = ServerId::new(serverid);
        if !user.servers.is_join(&serverid) {
            return Err("Server not exists".to_string());
        }
        let server = serverid
            .get_server(executor)
            .await
            .map_err(|_| "Bad ServerId".to_string())?;

        Ok((user, server))
    }

    pub async fn get_user_and_server(
        headers: &HeaderMap,
        serverid: String,
        executor: &DbConnection,
    ) -> Result<(User, Server), String> {
        let user = Self::get_user(headers, executor).await?;

        let server = ServerId::new(serverid)
            .get_server(executor)
            .await
            .map_err(|_| "Bad ServerId".to_string())?;

        Ok((user, server))
    }

    pub async fn get_user_and_server_and_check_if_joined_and_channel(
        headers: &HeaderMap,
        serverid: String,
        channelid: String,
        executor: &DbConnection,
    ) -> Result<(User, Server, Channel), String> {
        let (user, server) =
            Self::get_user_and_server_and_check_if_joined(headers, serverid, executor).await?;
        let channel_id = ChannelId::new(channelid);
        if server.channel.is_exists(channel_id.clone()) {
            return Err("Channel is not exists".to_string());
        }
        let channel = channel_id
            .get_channel(executor)
            .await
            .ok_or("Bad ChannelId".to_string())?;

        Ok((user, server, channel))
    }
}
