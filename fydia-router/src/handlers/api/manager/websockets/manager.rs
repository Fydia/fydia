use axum::async_trait;
use tokio::sync::oneshot;

use fydia_struct::{
    event::Event,
    instance::{Instance, RsaData},
    manager::{Manager, ManagerChannel},
    user::User,
};

use super::{ChannelMessage, WbChannel, WbManagerMessage, WbSender, WebsocketInner};

pub type WbManager = Manager<WebsocketInner>;

pub type WebsocketManagerChannel = ManagerChannel<WbManagerMessage>;

#[async_trait]
pub trait WbManagerChannelTrait {
    async fn get_channels_of_user(&self, user: User) -> Result<Vec<WbSender>, String>;
    async fn get_new_channel(&self, user: User) -> Option<WbChannel>;
    async fn remove(&self, user: User, wbsender: &WbSender) -> Result<(), ()>;
    async fn send(&self, msg: Event, user: Vec<User>) -> Result<(), String>;
    async fn send_with_origin_and_key(
        &self,
        msg: Event,
        user: Vec<User>,
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), String>;
}

#[async_trait]
impl WbManagerChannelTrait for WebsocketManagerChannel {
    async fn get_channels_of_user(&self, user: User) -> Result<Vec<WbSender>, String> {
        let (sender, receiver) = oneshot::channel::<Vec<WbSender>>();
        if let Err(e) = self.0.send(WbManagerMessage::Get(user, sender)) {
            error!(e.to_string());
        }

        match receiver.await {
            Ok(e) => Ok(e),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_new_channel(&self, user: User) -> Option<WbChannel> {
        let (sender, receiver) = oneshot::channel::<WbChannel>();
        if let Err(e) = self.0.send(WbManagerMessage::Insert(user, sender)) {
            error!(e.to_string());
        }

        receiver.await.ok()
    }

    async fn remove(&self, user: User, wbsender: &WbSender) -> Result<(), ()> {
        let (sender, receiver) = oneshot::channel::<Result<(), ()>>();
        if let Err(error) = self
            .0
            .send(WbManagerMessage::Remove(user, wbsender.clone(), sender))
        {
            error!(error.to_string());
        }

        receiver.await.unwrap_or_else(|_| Err(()))
    }

    async fn send(&self, msg: Event, user: Vec<User>) -> Result<(), String> {
        self.send_with_origin_and_key(msg, user, None, None).await
    }

    async fn send_with_origin_and_key(
        &self,
        msg: Event,
        user: Vec<User>,
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), String> {
        for mut i in user {
            i.drop_password();
            let channel = self.get_channels_of_user(i).await?;
            for i in channel {
                if let Err(e) = i.send(ChannelMessage::Message(Box::new(msg.clone()))) {
                    error!(e.to_string());
                }
            }
        }

        Ok(())
    }
}
