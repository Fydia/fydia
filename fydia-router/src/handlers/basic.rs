use fydia_sql::{
    impls::{channel::SqlChannelId, server::SqlServerId, token::SqlToken},
    sqlpool::DbConnection,
};
use http::HeaderMap;

use fydia_struct::{
    channel::{Channel, ChannelId},
    response::FydiaResponse,
    server::{Server, ServerId},
    user::{Token, User},
};

#[derive(Debug)]
pub struct BasicValues;

impl BasicValues {
    pub async fn get_user(
        headers: &HeaderMap,
        executor: &DbConnection,
    ) -> Result<User, FydiaResponse> {
        let token =
            Token::from_headervalue(headers).ok_or_else(|| FydiaResponse::new_error("No token"))?;
        token
            .get_user(executor)
            .await
            .ok_or_else(|| FydiaResponse::new_error("Wrong token"))
    }

    pub async fn get_user_and_server_and_check_if_joined<T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server), FydiaResponse> {
        let user = Self::get_user(headers, executor).await?;
        let serverid = ServerId::new(serverid);
        if !user.servers.is_join(&serverid) {
            return Err(FydiaResponse::new_error("Server not exists"));
        }
        let server = serverid
            .get_server(executor)
            .await
            .map_err(|_| FydiaResponse::new_error("Bad ServerId"))?;

        Ok((user, server))
    }

    pub async fn get_user_and_server<T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server), FydiaResponse> {
        let user = Self::get_user(headers, executor).await?;

        let server = ServerId::new(serverid)
            .get_server(executor)
            .await
            .map_err(|_| FydiaResponse::new_error("Bad ServerId"))?;

        Ok((user, server))
    }

    pub async fn get_user_and_server_and_check_if_joined_and_channel<T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        channelid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server, Channel), FydiaResponse> {
        let (user, server) =
            Self::get_user_and_server_and_check_if_joined(headers, serverid, executor).await?;
        let channel_id = ChannelId::new(channelid);
        if !server.channel.is_exists(&channel_id) {
            return Err(FydiaResponse::new_error("Channel is not exists"));
        }
        let channel = channel_id
            .get_channel(executor)
            .await
            .map_err(FydiaResponse::new_error)?;

        Ok((user, server, channel))
    }
}
