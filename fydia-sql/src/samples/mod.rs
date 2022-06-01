use fydia_struct::{
    channel::{Channel, ChannelId},
    instance::Instance,
    messages::{Date, Message},
    server::{Server, ServerId},
    user::User,
};

use crate::{
    impls::{message::SqlMessage, server::SqlServer, user::SqlUser},
    sqlpool::DbConnection,
};

/// Create default tables in database
///
/// # Errors
/// Return an error if:
/// * Database is unreachable
/// * Any tables errors
pub async fn insert_samples(db: &DbConnection) -> Result<(), String> {
    warn!("Insert Sample Values");

    let mut user = if let Some(user) =
        User::get_user_by_email_and_password("user@sample.com", "user", db).await
    {
        user
    } else {
        let mut user = User::new("user", "user@sample.com", "user", Instance::default())?;
        user.token = Some(String::from("default_token"));
        if let Err(error) = user.insert_user_and_update(db).await {
            error!("{error}");
        }

        user
    };

    let mut server = if let Ok(server) =
        Server::get_server_by_id(&ServerId::new("server_default_id"), db).await
    {
        info!("Server already exists");
        server
    } else {
        let mut server = Server::new("server_default", user.id.clone())?;

        server.id = ServerId::new("server_default_id");

        if let Err(error) = server.insert_server(db).await {
            error!("{error}");
        }

        server
    };

    user.update_from_database(db).await?;

    if !user.servers.is_join(&ServerId::new("server_default_id")) {
        if let Err(error) = server.join(&mut user, db).await {
            error!("{error}");
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
        )?;

        channel.id = ChannelId::new("channel_default_id");

        if let Err(error) = server.insert_channel(&channel, db).await {
            error!("{error}");
        }
    }

    if let Ok(message) =
        Message::get_messages_by_channel(ChannelId::new("channel_default_id"), db).await
    {
        if message.len() < 5 {
            for _ in 0..=5 {
                let message = Message::new(
                    "Message",
                    fydia_struct::messages::MessageType::TEXT,
                    false,
                    Date::now(),
                    user.clone(),
                    ChannelId::new("channel_default_id"),
                )?;

                if let Err(error) = message.insert_message(db).await {
                    error!("{error}");
                }
            }
        }
    }

    info!("Sample are insert in database");
    Ok(())
}
