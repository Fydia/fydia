use axum::async_trait;
use tokio::sync::oneshot;

use super::{ChannelMessage, WbChannel, WbManagerMessage, WbSender, WebsocketInner};
use fydia_struct::{
    event::Event,
    instance::{Instance, RsaData},
    manager::{Manager, ManagerChannel},
    user::UserId,
};

pub type WbManager = Manager<WebsocketInner>;

pub type WebsocketManagerChannel = ManagerChannel<WbManagerMessage>;

#[async_trait]
pub trait WbManagerChannelTrait {
    async fn get_channels_of_user(&self, user: &UserId) -> Result<Vec<WbSender>, String>;
    async fn get_new_channel(&self, user: &UserId) -> Option<WbChannel>;
    async fn remove(&self, user: &UserId, wbsender: &WbSender) -> Result<(), String>;
    async fn send(&self, msg: &Event, user: &[UserId]) -> Result<(), String>;
    async fn send_with_origin_and_key(
        &self,
        msg: &Event,
        user: &[UserId],
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), String>;
}

#[async_trait]
impl WbManagerChannelTrait for WebsocketManagerChannel {
    async fn get_channels_of_user(&self, user: &UserId) -> Result<Vec<WbSender>, String> {
        let (sender, receiver) = oneshot::channel::<Vec<WbSender>>();
        if let Err(e) = self.0.send(WbManagerMessage::Get(user.clone(), sender)) {
            error!("{}", e.to_string());
        }

        receiver.await.map_err(|error| error.to_string())
    }

    async fn get_new_channel(&self, user: &UserId) -> Option<WbChannel> {
        let (sender, receiver) = oneshot::channel::<Result<WbChannel, String>>();
        if let Err(e) = self.0.send(WbManagerMessage::Insert(user.clone(), sender)) {
            error!("{}", e.to_string());
        }

        receiver.await.ok()?.ok()
    }

    async fn remove(&self, user: &UserId, wbsender: &WbSender) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel::<Result<(), String>>();
        if let Err(error) = self.0.send(WbManagerMessage::Remove(
            user.clone(),
            wbsender.clone(),
            sender,
        )) {
            error!("{}", error.to_string());
        }

        receiver.await.map_err(|error| error.to_string())?
    }

    async fn send(&self, msg: &Event, user: &[UserId]) -> Result<(), String> {
        self.send_with_origin_and_key(msg, user, None, None).await
    }

    async fn send_with_origin_and_key(
        &self,
        msg: &Event,
        user: &[UserId],
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), String> {
        for i in user {
            let channels = self.get_channels_of_user(i).await?;
            for channel in channels {
                let msg = msg.clone();
                tokio::spawn(async move {
                    if let Err(e) = channel.send(ChannelMessage::Message(Box::new(msg))) {
                        error!("{}", e.to_string());
                    }
                });
            }
        }

        Ok(())
    }
}
