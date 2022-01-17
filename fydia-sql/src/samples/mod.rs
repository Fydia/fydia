use fydia_struct::{
    channel::{Channel, ChannelId},
    instance::Instance,
    messages::{Message, SqlDate},
    server::{Server, ServerId},
    user::User,
};

use crate::{
    impls::{message::SqlMessage, server::SqlServer, user::SqlUser},
    sqlpool::DbConnection,
};

pub async fn insert_samples(db: &DbConnection) {
    warn!("Insert Sample Values");

    let mut user = match User::get_user_by_email_and_password(
        "user@sample.com".to_string(),
        "user".to_string(),
        &db,
    )
    .await
    {
        Some(user) => user,
        None => {
            let mut user = User::new("user", "user@sample.com", "user", Instance::default());
            if let Err(error) = user.insert_user_and_update(db).await {
                error!(error);
            }

            user
        }
    };

    let mut server = if let Ok(server) =
        Server::get_server_by_id(ServerId::new("server_default_id"), db).await
    {
        info!("Server already exists");
        server
    } else {
        let mut server = Server::new("server_default", user.id.clone());
        server.id = ServerId::new("server_default_id");

        if let Err(error) = server.insert_server(db).await {
            error!(error);
        }

        server
    };

    if !user.servers.is_join(ServerId::new("server_default_id")) {
        if let Err(error) = server.join(&mut user, db).await {
            error!(error);
        }
    }

    if !server
        .channel
        .is_exists(ChannelId::new("channel_default_id"))
    {
        let mut channel = Channel::new(
            "channel_default",
            "channel_default",
            fydia_struct::channel::ChannelType::Text,
        );
        channel.id = ChannelId::new("channel_default_id");
        if let Err(error) = server.insert_channel(channel.clone(), db).await {
            error!(error);
        }
    }
    if let Ok(message) =
        Message::get_messages_by_channel(ChannelId::new("channel_default_id"), db).await
    {
        if message.len() < 5 {
            for _ in 0..=5 {
                if let Err(error) = Message::new(
                    "Message",
                    fydia_struct::messages::MessageType::TEXT,
                    false,
                    SqlDate::now(),
                    user.clone(),
                    ChannelId::new("channel_default_id"),
                )
                .insert_message(db)
                .await
                {
                    error!(error);
                }
            }
        }
    }

    success!("Sample are insert in database");
}
