use crate::ServerState;
use axum::{
    extract::{FromRequest, FromRequestParts, RawPathParams},
    http::{header::CONTENT_TYPE, Request},
};
use fydia_sql::{
    impls::{
        channel::SqlChannelId, message::SqlMessage, role::SqlRoles, server::SqlServerId,
        token::SqlToken, user::SqlUser,
    },
    sqlpool::DbConnection,
};
use fydia_struct::{
    channel::{Channel, ChannelError, ChannelId},
    instance::RsaData,
    messages::Message,
    response::FydiaResponse,
    roles::Role,
    server::{Server, ServerError, ServerId},
    user::{Token, User},
};
use fydia_utils::async_trait;
use mime::Mime;
use std::{marker::PhantomData, str::FromStr, sync::Arc};

#[derive(Debug)]
pub struct ContentType(pub mime::Mime, pub String);

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for ContentType {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let content_type = parts
            .headers
            .get(CONTENT_TYPE)
            .ok_or_else(|| FydiaResponse::TextError("No Content-Type header found"))?
            .to_str()
            .map_err(|error| {
                error!("{error}");
                FydiaResponse::TextError("Content-Type error")
            })?;

        let mime_type = Mime::from_str(content_type).map_err(|error| {
            error!("{error}");
            FydiaResponse::TextError("Bad Content-Type")
        })?;

        Ok(Self(mime_type, content_type.to_string()))
    }
}

macro_rules! create_from_state {
    ($name:ident, $type:ty, $value:ident) => {
        #[derive(Debug)]
        pub struct $name(pub $type);

        #[async_trait::async_trait]
        impl FromRequestParts<ServerState> for $name {
            type Rejection = FydiaResponse;

            async fn from_request_parts(
                _: &mut axum::http::request::Parts,
                state: &ServerState,
            ) -> Result<Self, Self::Rejection> {
                Ok(Self(state.$value.clone()))
            }
        }
    };
}

create_from_state!(WebsocketManager, Arc<WebsocketManagerChannel>, wbsocket);
create_from_state!(Rsa, Arc<RsaData>, rsa);
create_from_state!(Database, DbConnection, database);
create_from_state!(TypingManager, Arc<TypingManagerChannel>, typing);

#[derive(Debug)]
struct UrlGetter<T: UrlName>(String, PhantomData<T>);

impl<T: UrlName> UrlGetter<T> {
    pub fn get_key() -> String {
        T::URL_KEY.to_string()
    }
}

trait UrlName: Sized {
    const URL_KEY: &'static str;
}

#[async_trait::async_trait]
impl<T: UrlName> FromRequestParts<ServerState> for UrlGetter<T> {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let raw = RawPathParams::from_request_parts(parts, state)
            .await
            .map_err(|f| FydiaResponse::StringError(Box::new(f.to_string())))?;

        for (key, value) in raw.iter() {
            if key == Self::get_key() {
                return Ok(Self(value.to_string(), PhantomData));
            }
        }

        Err(FydiaResponse::TextError("No message id"))
    }
}
#[derive(Debug)]
pub struct RoleFromId(pub Role);

impl UrlName for RoleFromId {
    const URL_KEY: &'static str = "roleid";
}

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for RoleFromId {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let UserFromToken(user) = UserFromToken::from_request_parts(parts, state).await?;
        let ChannelFromId(channel) = ChannelFromId::from_request_parts(parts, state).await?;
        let ServerJoinedFromId(server) =
            ServerJoinedFromId::from_request_parts(parts, state).await?;

        let UrlGetter(roleid, _) =
            UrlGetter::<RoleFromId>::from_request_parts(parts, state).await?;

        if !user
            .permission_of_channel(&channel.id, &state.database)
            .await?
            .calculate(Some(channel.id.clone()))?
            .is_admin()
        {
            return Err(FydiaResponse::TextError("Unknow channel"));
        }
        let roleid = roleid.as_str().parse()?;
        let role = Role::by_id(roleid, &server.id, &state.database).await?;

        Ok(Self(role))
    }
}

struct MessageId(pub String);

impl UrlName for MessageId {
    const URL_KEY: &'static str = "messageid";
}
#[derive(Debug)]
pub struct MessageFromId(pub Message);

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for MessageFromId {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let UserFromToken(user) = UserFromToken::from_request_parts(parts, state).await?;
        let ChannelFromId(channel) = ChannelFromId::from_request_parts(parts, state).await?;

        let UrlGetter(url_param, _) =
            UrlGetter::<MessageId>::from_request_parts(parts, state).await?;

        if !user
            .permission_of_channel(&channel.id, &state.database)
            .await?
            .calculate(Some(channel.id.clone()))?
            .can_read()
        {
            return Err(ChannelError::CannotGetById.into());
        }

        let message = Message::by_id(&url_param, &state.database).await?;

        Ok(Self(message))
    }
}

use super::{
    api::manager::{typing::TypingManagerChannel, websockets::manager::WebsocketManagerChannel},
    get_json, get_json_value_from_body,
};

#[derive(Debug)]
pub struct ChannelFromId(pub Channel);

impl UrlName for ChannelFromId {
    const URL_KEY: &'static str = "channelid";
}

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for ChannelFromId {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let ServerJoinedFromId(server) =
            ServerJoinedFromId::from_request_parts(parts, state).await?;

        let UrlGetter(channelid, _) =
            UrlGetter::<ChannelFromId>::from_request_parts(parts, state).await?;

        let channel = ChannelId::new(channelid).channel(&state.database).await?;

        if !server.channel.is_exists(&channel.id) {
            return Err(ChannelError::CannotGetById.into());
        }

        Ok(Self(channel))
    }
}

#[derive(Debug)]
pub struct ServerJoinedFromId(pub Server);

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for ServerJoinedFromId {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let UserFromToken(user) = UserFromToken::from_request_parts(parts, state).await?;
        let ServerFromId(server) = ServerFromId::from_request_parts(parts, state).await?;

        if !user.servers.is_join(&server.id) {
            return Err(ServerError::CannotGetById.into());
        }

        Ok(Self(server))
    }
}

#[derive(Debug)]
pub struct ServerFromId(pub Server);

impl UrlName for ServerFromId {
    const URL_KEY: &'static str = "serverid";
}

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for ServerFromId {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let UrlGetter(serverid, _) =
            UrlGetter::<ServerFromId>::from_request_parts(parts, state).await?;

        let server = ServerId::new(serverid).get(&state.database).await?;

        Ok(ServerFromId(server))
    }
}

#[derive(Debug)]
pub struct UserFromJson(pub User);

#[async_trait::async_trait]
impl FromRequest<ServerState, axum::body::Body> for UserFromJson {
    type Rejection = FydiaResponse;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let body = String::from_request(req, state)
            .await
            .map_err(|f| FydiaResponse::StringError(Box::new(f.to_string())))?;

        let json = get_json_value_from_body(&body)?;

        let email = get_json("email", &json)?;
        let password = get_json("password", &json)?;

        let user = User::by_email_and_password(email, password, &state.database).await?;

        Ok(UserFromJson(user))
    }
}

#[derive(Debug)]
pub struct UserFromToken(pub User);

#[async_trait::async_trait]
impl FromRequestParts<ServerState> for UserFromToken {
    type Rejection = FydiaResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let token = Token::from_headervalue(&parts.headers);

        let user = token.get_user(&state.database).await?;

        Ok(UserFromToken(user))
    }
}
