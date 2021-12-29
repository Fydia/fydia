use axum::async_trait;
use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::EventContent;
use fydia_struct::manager::ManagerReceiverTrait;
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::server::ServerId;
use fydia_struct::{event::Event, user::User};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio::sync::oneshot::Sender as OSSender;

use self::manager::{WbManagerChannelTrait, WebsocketManagerChannel};

pub mod manager;
pub mod messages;

pub type WbChannel = (WbSender, WbReceiver);
pub type WbReceiver = Receiver<ChannelMessage>;
pub type WbSender = Sender<ChannelMessage>;
#[derive(Debug, Clone)]
pub struct WbStruct(User, Vec<WbSender>);

impl WbStruct {
    pub fn new(user: User) -> Self {
        Self(
            User {
                id: user.id,
                name: user.name,
                ..Default::default()
            },
            Vec::new(),
        )
    }

    pub fn get_senders(&mut self) -> Vec<WbSender> {
        self.1.to_vec()
    }

    pub fn get_sender(&mut self, index: usize) -> Option<WbSender> {
        self.1.get(index).cloned()
    }

    pub fn insert_channel(&mut self, wbsender: WbSender) -> WbSender {
        self.1.push(wbsender.clone());
        wbsender
    }

    pub fn is_same_user(&self, user: &User) -> bool {
        self.0
            == User {
                id: user.id,
                name: user.name.clone(),
                ..Default::default()
            }
    }
}

#[derive(Debug)]
pub struct WebsocketInner {
    wb_channel: Mutex<Vec<WbStruct>>,
}

#[async_trait]
impl ManagerReceiverTrait for WebsocketInner {
    type Message = WbManagerMessage;
    async fn on_receiver(&mut self, message: Self::Message) {
        match message {
            WbManagerMessage::Get(user, callback) => {
                if callback.send(self.get_user(&user).1).is_err() {
                    error!("Can't send");
                };
            }
            WbManagerMessage::Insert(user, callback) => {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ChannelMessage>();

                self.insert_channel(&user, sender.clone());

                if callback.send((sender, receiver)).is_err() {
                    error!("Error on insert");
                };
            }
            WbManagerMessage::Remove(user, wbsender, callback) => {
                if callback
                    .send(self.remove_sender(&user, &wbsender).await)
                    .is_err()
                {
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

impl Default for WebsocketInner {
    fn default() -> Self {
        Self {
            wb_channel: Mutex::new(Vec::new()),
        }
    }
}

impl WebsocketInner {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn get_user(&self, user: &User) -> WbStruct {
        let user_index = self.get_user_index(user);
        let user = &self.wb_channel.lock()[user_index];
        user.clone()
    }

    pub fn get_user_index(&self, user: &User) -> usize {
        for (n, i) in self.wb_channel.lock().iter().enumerate() {
            if i.is_same_user(user) {
                return n;
            }
        }

        let mut wb_channel = self.wb_channel.lock();
        wb_channel.push(WbStruct::new(user.clone()));
        wb_channel.len() - 1
    }

    pub fn get_sender_index(&self, user: &User, sender: &WbSender) -> Option<usize> {
        let user_index = self.get_user_index(user);
        let wb_struct = &self.wb_channel.lock()[user_index];
        for (n, i) in wb_struct.1.iter().enumerate() {
            if i.same_channel(sender) {
                return Some(n);
            }
        }

        None
    }

    pub fn get_sender_index_and_user_index(
        &self,
        user: &User,
        sender: &WbSender,
    ) -> Option<(usize, usize)> {
        let user_index = self.get_user_index(user);
        let wb_struct = &self.wb_channel.lock()[user_index];
        for (n, i) in wb_struct.1.iter().enumerate() {
            if i.same_channel(sender) {
                return Some((n, user_index));
            }
        }

        None
    }

    pub fn get_channel(&mut self, user: &User, index: usize) -> Option<WbSender> {
        if let Some(sender) = self.get_user(user).1.get(index) {
            return Some(sender.clone());
        }

        None
    }

    pub fn insert_channel(&mut self, user: &User, channel: WbSender) {
        let user_nth = self.get_user_index(user);
        if let Some(user) = self.wb_channel.get_mut().iter_mut().nth(user_nth) {
            user.1.push(channel)
        }
    }

    pub fn remove(&mut self, index: usize) {
        let mut wbchannel = self.wb_channel.lock();
        wbchannel.swap_remove(index);
        wbchannel.shrink_to_fit();
    }

    pub async fn remove_sender(&mut self, user: &User, wbsender: &WbSender) -> Result<(), ()> {
        let mut user = user.clone();
        user.password = None;

        if let Some((index_channel, index_user)) =
            self.get_sender_index_and_user_index(&user, wbsender)
        {
            let mut wblist = self.wb_channel.lock();
            let wbuser = &mut wblist[index_user].1;
            wbuser.swap_remove(index_channel);

            if wbuser.is_empty() {
                wblist.swap_remove(index_user);
            }

            wblist.shrink_to_fit();

            return Ok(());
        }
        Err(())
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
    Get(User, OSSender<Vec<WbSender>>),
    GetWithIndex(User, usize, OSSender<Option<WbSender>>),
    Insert(User, OSSender<WbChannel>),
    Remove(User, WbSender, OSSender<Result<(), ()>>),
}

pub async fn test_message(
    Extension(websocket_manager_channel): Extension<Arc<WebsocketManagerChannel>>,
) -> impl IntoResponse {
    let instant = Instant::now();
    if let Ok(getted_websocket) = websocket_manager_channel
        .get_channels_of_user(User::default())
        .await
    {
        for i in &getted_websocket {
            if let Err(e) = i.send(ChannelMessage::Message(Box::new(Event::new(
                ServerId::new(String::new()),
                EventContent::Message {
                    content: Box::from(Message::new(
                        String::new(),
                        MessageType::TEXT,
                        false,
                        SqlDate::now(),
                        User::default(),
                        ChannelId::default(),
                    )),
                },
            )))) {
                println!("{}", e.to_string())
            };
        }
        format!(
            "{}µs => {:?}",
            instant.elapsed().as_micros(),
            getted_websocket
        )
    } else {
        format!("{}µs => Error", instant.elapsed().as_micros(),)
    }
}
