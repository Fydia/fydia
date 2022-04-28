use std::collections::HashMap;

use axum::async_trait;
use fydia_struct::manager::ManagerReceiverTrait;
use fydia_struct::{event::Event, user::UserInfo};
use parking_lot::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio::sync::oneshot::Sender as OSSender;

pub mod manager;
pub mod messages;

pub type WbChannel = (WbSender, WbReceiver);
pub type WbReceiver = Receiver<ChannelMessage>;
pub type WbSender = Sender<ChannelMessage>;

#[derive(Debug)]
pub struct WebsocketInner {
    wb_channel: RwLock<HashMap<UserInfo, Vec<WbSender>>>,
}

impl Default for WebsocketInner {
    fn default() -> Self {
        Self {
            wb_channel: RwLock::new(HashMap::new()),
        }
    }
}

impl WebsocketInner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_sender_index(&self, user: &UserInfo, sender: &WbSender) -> Option<usize> {
        let wb_channel = self.wb_channel.read();
        let wb_struct = wb_channel.get(user)?;
        for (n, i) in wb_struct.iter().enumerate() {
            if i.same_channel(sender) {
                return Some(n);
            }
        }

        None
    }

    pub fn get_channels(&mut self, user: &UserInfo) -> Vec<WbSender> {
        match self.wb_channel.read().get(user) {
            Some(wbsenders) => wbsenders.clone(),
            None => Vec::new(),
        }
    }

    pub fn get_channel(&mut self, user: &UserInfo, index: usize) -> Option<WbSender> {
        self.wb_channel.read().get(user)?.get(index).cloned()
    }

    pub fn insert_user(&mut self, user: &UserInfo) {
        let mut wbchannel = self.wb_channel.write();
        if !wbchannel.contains_key(user) {
            wbchannel.insert(user.clone(), Vec::new());
        }
    }

    /// Inser a websocket channel
    ///
    /// # Errors
    /// Return an error if:
    /// * User doesn't exist
    pub fn insert_channel(&mut self, user: &UserInfo) -> Result<WbChannel, String> {
        let mut wbchannel = self.wb_channel.write();

        let user = wbchannel
            .get_mut(user)
            .ok_or_else(|| String::from("User not in HashMap"))?;

        let channel = tokio::sync::mpsc::unbounded_channel::<ChannelMessage>();

        user.push(channel.0.clone());

        Ok(channel)
    }

    /// Remove a websocket channel
    ///
    /// # Errors
    /// Return an error if:
    /// * wbsender doesn't exist
    /// * User doesn't exist
    pub fn remove(&mut self, user: &UserInfo, websocket_channel: &WbSender) -> Result<(), String> {
        let index = self
            .get_sender_index(user, websocket_channel)
            .ok_or_else(|| String::from("Wbsender not in Vec"))?;

        let mut wbchannel = self.wb_channel.write();
        let channel = wbchannel
            .get_mut(user)
            .ok_or_else(|| String::from("User not in hashmap"))?;

        channel.remove(index);

        if channel.is_empty() {
            wbchannel.remove(user);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    WebsocketMessage(axum::extract::ws::Message),
    Message(Box<Event>),
    Kill,
}

#[derive(Debug)]
pub enum WbManagerMessage {
    Get(UserInfo, OSSender<Vec<WbSender>>),
    GetWithIndex(UserInfo, usize, OSSender<Option<WbSender>>),
    Insert(UserInfo, OSSender<Result<WbChannel, String>>),
    Remove(UserInfo, WbSender, OSSender<Result<(), String>>),
}

#[async_trait]
impl ManagerReceiverTrait for WebsocketInner {
    type Message = WbManagerMessage;
    async fn on_receiver(&mut self, message: Self::Message) {
        match message {
            WbManagerMessage::Get(user, callback) => {
                if callback.send(self.get_channels(&user)).is_err() {
                    error!("Can't send");
                };
            }
            WbManagerMessage::Insert(user, callback) => {
                self.insert_user(&user);
                if callback.send(self.insert_channel(&user)).is_err() {
                    error!("Error on insert");
                };
            }
            WbManagerMessage::Remove(user, wbsender, callback) => {
                if callback.send(self.remove(&user, &wbsender)).is_err() {
                    error!("Can't Remove");
                }
            }
            WbManagerMessage::GetWithIndex(user, index, callback) => {
                if callback.send(self.get_channel(&user, index)).is_err() {
                    error!("Can't send");
                };
            }
        }
    }
}
