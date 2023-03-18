use fydia_struct::{
    channel::{Channel, ChannelId},
    instance::Instance,
    messages::{Date, Message},
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
#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
pub async fn insert_samples(db: &DbConnection) -> Result<(), String> {
    warn!("Insert Sample Values");

    let mut user =
        if let Ok(user) = User::by_token(&Token::new("default_token".to_string()), db).await {
            user
        } else {
            let mut user = User::new("user", "user@sample.com", "user", Instance::default())?;

            user.token = Token::new(String::from("default_token"));

            user = user.insert(db).await.unwrap();

            user
        };

    let mut server =
        if let Ok(server) = Server::by_id(&ServerId::new("server_default_id"), db).await {
            info!("Server already exists");
            server
        } else {
            let mut server = Server::new("server_default", user.id.clone()).unwrap();

            server.id = ServerId::new("server_default_id");

            if let Err(error) = server.insert(db).await {
                error!("{}", error);
            }

            server
        };

    user.update_from_database(db).await.unwrap();

    if !user.servers.is_join(&ServerId::new("server_default_id")) {
        if let Err(error) = server.join(&mut user, db).await {
            error!("{}", error);
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
        .unwrap();

        channel.id = ChannelId::new("channel_default_id");

        if let Err(error) = server.insert_channel(&channel, db).await {
            error!("{}", error);
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
                .unwrap();

                if let Err(error) = message.insert(db).await {
                    error!("{}", error);
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

    role.insert(db).await.unwrap();

    role.add_user(&user.id, db).await.unwrap();

    info!("Sample are insert in database");

    Ok(())
}
