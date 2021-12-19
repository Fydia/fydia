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
    async fn send(
        &self,
        msg: Event,
        user: Vec<User>,
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), ()>;
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

        match receiver.await {
            Ok(e) => return Some(e),
            Err(e) => {
                error!(e.to_string());
            }
        };

        None
    }

    async fn remove(&self, user: User, wbsender: &WbSender) -> Result<(), ()> {
        let (sender, receiver) = oneshot::channel::<Result<(), ()>>();
        if let Err(e) = self
            .0
            .send(WbManagerMessage::Remove(user, wbsender.clone(), sender))
        {
            error!(e.to_string());
        }

        if let Ok(res) = receiver.await {
            return res;
        }

        Err(())
    }

    async fn send(
        &self,
        msg: Event,
        user: Vec<User>,
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), ()> {
        for mut i in user {
            i.drop_password();
            if let Ok(wbstruct) = self.get_channels_of_user(i).await {
                for i in wbstruct {
                    if let Err(e) = i.send(ChannelMessage::Message(Box::new(msg.clone()))) {
                        error!(e.to_string());
                    };
                }
            } else {
                return Err(());
            }
        }

        Ok(())
    }
}
