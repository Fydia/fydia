use fydia_struct::{
    channel::{Channel, ChannelId},
    instance::Instance,
    messages::{Date, Message},
    response::{FydiaResponse, MapError},
    roles::Role,
    server::{Server, ServerId},
    user::{Token, User},
    utils::Id,
};

use crate::{
    impls::{message::SqlMessage, role::SqlRoles, server::SqlServer, user::SqlUser},
    sqlpool::DbConnection,
};

/// Create default tables in database
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Any tables errors
pub async fn insert_samples<'a>(db: &DbConnection) -> Result<(), FydiaResponse<'a>> {
    warn!("Insert Sample Values");

    let mut user = if let Some(user) = User::by_token(&Token("default_token".to_string()), db).await
    {
        user
    } else {
        let mut user = User::new("user", "user@sample.com", "user", Instance::default())
            .error_to_fydiaresponse()?;

        user.token = Some(String::from("default_token"));

        user = user.insert(db).await?;

        user
    };

    let mut server = if let Ok(server) =
        Server::by_id(&ServerId::new("server_default_id"), db).await
    {
        info!("Server already exists");
        server
    } else {
        let mut server = Server::new("server_default", user.id.clone()).error_to_fydiaresponse()?;

        server.id = ServerId::new("server_default_id");

        if let Err(error) = server.insert(db).await {
            error!("{}", error.get_string());
        }

        server
    };

    user.update_from_database(db).await?;

    if !user.servers.is_join(&ServerId::new("server_default_id")) {
        if let Err(error) = server.join(&mut user, db).await {
            error!("{}", error.get_string());
        }
    }

    if !server
        .channel
        .is_exists(&ChannelId::new("channel_default_id"))
    {
        let mut channel = Channel::new_with_serverid(
            "channel_default",
            "channel_default",
            ServerId::new("server_default_id"),
            fydia_struct::channel::ChannelType::Text,
        )
        .error_to_fydiaresponse()?;

        channel.id = ChannelId::new("channel_default_id");

        if let Err(error) = server.insert_channel(&channel, db).await {
            error!("{}", error.get_string());
        }
    }

    if let Ok(message) = Message::by_channel(ChannelId::new("channel_default_id"), db).await {
        if message.len() < 5 {
            for _ in 0..=5 {
                let message = Message::new(
                    "Message",
                    fydia_struct::messages::MessageType::TEXT,
                    false,
                    Date::now(),
                    user.clone(),
                    ChannelId::new("channel_default_id"),
                )
                .error_to_fydiaresponse()?;

                if let Err(error) = message.insert(db).await {
                    error!("{}", error.get_string());
                }
            }
        }
    }

    let mut role = Role {
        id: Id::Unset,
        server_id: server.id,
        name: String::from("default_role"),
        color: String::from("ffffff"),
        server_permission: 4,
    };

    role.insert(db).await?;

    role.add_user(&user.id, db).await?;

    info!("Sample are insert in database");

    Ok(())
}
