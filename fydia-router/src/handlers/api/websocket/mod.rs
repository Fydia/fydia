use flume::{Receiver, Sender};
use fydia_dispatcher::message::send::encrypt_message;
use fydia_struct::event::Event;
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::user::User;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub mod messages;

#[derive(Debug, Clone)]
pub struct Websockets {
    channels: Arc<Mutex<WbList>>,
}

impl Default for Websockets {
    fn default() -> Self {
        Self::new()
    }
}

impl Websockets {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(WbList::new())),
        }
    }

    pub async fn get_channels_clone(&self) -> WbList {
        self.channels.lock().await.clone()
    }

    pub async fn get_channels(&self) -> MutexGuard<'_, WbList> {
        self.channels.lock().await
    }

    pub async fn insert(&self, wbuser: &WbUser) {
        let mut res = self.channels.lock().await;
        res.insert(wbuser);
    }
    pub async fn remove_wbuser(&self, wbuser: &WbUser) -> Result<(), ()> {
        self.channels.lock().await.remove_wbuser(wbuser)
    }
    pub async fn remove(&self, id: u32) -> Result<(), ()> {
        self.channels.lock().await.remove(id)
    }

    pub async fn send(
        &self,
        msg: &Event,
        user: Vec<User>,
        keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) {
        self.get_channels().await.0.par_iter().for_each(|i| {
            if user.contains(&i.user) {
                if i.user.instance.domain == "localhost" || i.user.instance.domain.is_empty() {
                    if let Err(e) = i
                        .channel
                        .0
                        .send(ChannelMessage::Message(Box::new(msg.clone())))
                    {
                        error!(format!("Cannot send message : {}", e.to_string()));
                    }
                } else if let Some(rsa) = keys {
                    if let Ok(public_key) = i.user.instance.get_public_key() {
                        let _encrypt_message = encrypt_message(rsa, public_key, msg.clone());
                        //send_message(rsa, origin,  i.user.instance.get_public_key().unwrap(), message, instances);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct WbList(pub Vec<WbUser>);

impl WbList {
    pub fn new() -> Self {
        Self { 0: Vec::new() }
    }
    pub fn insert(&mut self, wbuser: &WbUser) {
        self.0.push(wbuser.clone());
    }
    pub fn get_user_by_id(&self, id: i32) -> Option<WbUser> {
        for i in &self.0 {
            if i.user.id == id {
                return Some(i.clone());
            }
        }

        None
    }

    pub fn get_user_by_token(&self, token: String) -> Option<WbUser> {
        for i in &self.0 {
            if i.user.token == Some(token.clone()) {
                return Some(i.clone());
            }
        }

        None
    }

    pub async fn send(
        &mut self,
        msg: &Event,
        user: Vec<User>,
        keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), ()> {
        self.0.par_iter().for_each(|i| {
            if user.contains(&i.user) {
                if i.user.instance.domain == "localhost" || i.user.instance.domain.is_empty() {
                    if let Err(e) = i
                        .channel
                        .0
                        .send(ChannelMessage::Message(Box::new(msg.clone())))
                    {
                        error!(format!("Cannot send message : {}", e.to_string()));
                    }
                } else if let Some(rsa) = keys {
                    if let Ok(public_key) = i.user.instance.get_public_key() {
                        let _encrypt_message = encrypt_message(rsa, public_key, msg.clone());
                        //send_message(rsa, origin,  i.user.instance.get_public_key().unwrap(), message, instances);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn remove(&mut self, id: u32) -> Result<(), ()> {
        let a = &self.0;
        for (n, i) in a.iter().enumerate() {
            if i.id == id {
                self.0.remove(n);
                return Ok(());
            }
        }

        Err(())
    }

    pub fn remove_wbuser(&mut self, wbuser: &WbUser) -> Result<(), ()> {
        let a = &self.0;
        for (n, i) in a.iter().enumerate() {
            if i == wbuser {
                self.0.remove(n);
                return Ok(());
            }
        }

        Err(())
    }
}

impl Default for WbList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WbUser {
    pub id: u32,
    pub channel: (Sender<ChannelMessage>, Receiver<ChannelMessage>),
    pub user: User,
}

impl Default for WbUser {
    fn default() -> Self {
        Self {
            id: Default::default(),
            channel: flume::unbounded(),
            user: Default::default(),
        }
    }
}

impl PartialEq for WbUser {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.user == other.user
    }
}

impl WbUser {
    pub fn new(
        id: u32,
        channel: (Sender<ChannelMessage>, Receiver<ChannelMessage>),
        user: User,
    ) -> Self {
        let mut user = user.clone();
        user.password = None;
        Self { id, channel, user }
    }
}

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    WebsocketMessage(axum::extract::ws::Message),
    Message(Box<Event>),
    Kill,
}
