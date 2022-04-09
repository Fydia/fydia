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
    pub async fn get_user<'a>(
        headers: &HeaderMap,
        executor: &DbConnection,
    ) -> Result<User, FydiaResponse<'a>> {
        let token = Token::from_headervalue(headers).ok_or(FydiaResponse::TextError("No token"))?;

        token
            .get_user(executor)
            .await
            .ok_or(FydiaResponse::TextError("Wrong token"))
    }

    pub async fn get_user_and_server_and_check_if_joined<'a, T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server), FydiaResponse<'a>> {
        let user = Self::get_user(headers, executor).await?;

        let server = ServerId::new(serverid)
            .get_server(executor)
            .await
            .map_err(|_| FydiaResponse::TextError("Server not exists"))?;

        if !user.servers.is_join(&server.id) {
            return Err(FydiaResponse::TextError("Server not exists"));
        }

        Ok((user, server))
    }

    pub async fn get_user_and_server<'a, T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server), FydiaResponse<'a>> {
        let user = Self::get_user(headers, executor).await?;

        let server = ServerId::new(serverid)
            .get_server(executor)
            .await
            .map_err(|_| FydiaResponse::TextError("Bad ServerId"))?;

        Ok((user, server))
    }

    pub async fn get_user_and_server_and_check_if_joined_and_channel<'a, T: Into<String>>(
        headers: &HeaderMap,
        serverid: T,
        channelid: T,
        executor: &DbConnection,
    ) -> Result<(User, Server, Channel), FydiaResponse<'a>> {
        let (user, server) =
            Self::get_user_and_server_and_check_if_joined(headers, serverid, executor).await?;

        let channel = ChannelId::new(channelid)
            .get_channel(executor)
            .await
            .map_err(FydiaResponse::StringError)?;

        if !server.channel.is_exists(&channel.id) {
            return Err(FydiaResponse::TextError("Channel is not exists"));
        }

        Ok((user, server, channel))
    }
}
