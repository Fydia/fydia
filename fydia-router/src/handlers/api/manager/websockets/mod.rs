use axum::async_trait;
use fydia_struct::manager::ManagerReceiverTrait;
use fydia_struct::{event::Event, user::UserInfo};
use parking_lot::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender};
use tokio::sync::oneshot::Sender as OSSender;

pub mod manager;
pub mod messages;

pub type WbChannel = (WbSender, WbReceiver);
pub type WbReceiver = Receiver<ChannelMessage>;
pub type WbSender = Sender<ChannelMessage>;
#[derive(Debug, Clone)]
pub struct WbStruct(UserInfo, Vec<WbSender>);

impl WbStruct {
    pub fn new(user: UserInfo) -> Self {
        Self(user, Vec::new())
    }

    pub fn get_senders(&mut self) -> Vec<WbSender> {
        self.1.to_vec()
    }

    pub fn get_sender(&mut self, index: usize) -> Option<WbSender> {
        self.1.get(index).cloned()
    }

    pub fn insert_channel(&mut self) -> WbChannel {
        let channel = tokio::sync::mpsc::unbounded_channel::<ChannelMessage>();
        self.1.push(channel.0.clone());
        channel
    }

    pub fn is_same_user(&self, user: &UserInfo) -> bool {
        println!("cmp : {:#?} / cmp_user: {:#?}", &self.0, user);
        self.0.eq(user)
    }
}

#[derive(Debug)]
pub struct WebsocketInner {
    wb_channel: Mutex<Vec<WbStruct>>,
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

    pub fn get_user(&self, user: &UserInfo) -> Option<WbStruct> {
        let user_index = self.get_user_index(user)?;
        self.wb_channel.lock().get(user_index).cloned()
    }

    pub fn get_user_index(&self, user: &UserInfo) -> Option<usize> {
        for (n, i) in self.wb_channel.lock().iter().enumerate() {
            if i.is_same_user(user) {
                return Some(n);
            }
        }

        None
    }

    pub fn get_sender_index(&self, user: &UserInfo, sender: &WbSender) -> Option<usize> {
        let user_index = self.get_user_index(user)?;
        let wb_channel = self.wb_channel.lock();
        let wb_struct = wb_channel.get(user_index)?;
        for (n, i) in wb_struct.1.iter().enumerate() {
            if i.same_channel(sender) {
                return Some(n);
            }
        }

        None
    }

    pub fn get_sender_index_and_user_index(
        &self,
        user: &UserInfo,
        sender: &WbSender,
    ) -> Option<(usize, usize)> {
        let user_index = self.get_user_index(user)?;
        let wb_channel = self.wb_channel.lock();
        let wb_struct = wb_channel.get(user_index)?;
        for (n, i) in wb_struct.1.iter().enumerate() {
            if i.same_channel(sender) {
                return Some((n, user_index));
            }
        }

        None
    }

    pub fn get_channel(&mut self, user: &UserInfo, index: usize) -> Option<WbSender> {
        self.get_user(user)?.1.get(index).cloned()
    }

    pub fn insert_user(&mut self, user: &UserInfo) -> Result<(), String> {
        if self.get_user_index(user).is_some() {
            return Err("User already exists".to_string());
        }

        self.wb_channel.lock().push(WbStruct::new(user.clone()));

        Ok(())
    }

    pub fn insert_channel(&mut self, user: &UserInfo) -> Result<WbChannel, String> {
        let user_nth = self
            .get_user_index(user)
            .ok_or_else(|| "No User".to_string())?;
        if let Some(user) = self.wb_channel.get_mut().iter_mut().nth(user_nth) {
            return Ok(user.insert_channel());
        }

        Err("No User".to_string())
    }

    pub fn remove(&mut self, index: usize) {
        let mut wbchannel = self.wb_channel.lock();
        wbchannel.swap_remove(index);
        wbchannel.shrink_to_fit();
    }

    pub async fn remove_sender(&mut self, user: &UserInfo, wbsender: &WbSender) -> Result<(), ()> {
        let (index_channel, index_user) = self
            .get_sender_index_and_user_index(user, wbsender)
            .ok_or(())?;

        let mut wblist = self.wb_channel.lock();
        let wbuser = &mut wblist[index_user].1;
        wbuser.swap_remove(index_channel);

        if wbuser.is_empty() {
            wblist.swap_remove(index_user);
        }

        wblist.shrink_to_fit();

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
    Get(UserInfo, OSSender<Option<Vec<WbSender>>>),
    GetWithIndex(UserInfo, usize, OSSender<Option<WbSender>>),
    Insert(UserInfo, OSSender<Result<WbChannel, String>>),
    Remove(UserInfo, WbSender, OSSender<Result<(), ()>>),
}

#[async_trait]
impl ManagerReceiverTrait for WebsocketInner {
    type Message = WbManagerMessage;
    async fn on_receiver(&mut self, message: Self::Message) {
        match message {
            WbManagerMessage::Get(user, callback) => {
                if callback
                    .send(
                        self.get_user(&user)
                            .map(|mut wbstruct| wbstruct.get_senders()),
                    )
                    .is_err()
                {
                    error!("Can't send");
                };
            }
            WbManagerMessage::Insert(user, callback) => {
                if let Err(error) = self.insert_user(&user) {
                    error!(error);
                }

                let channel = self.insert_channel(&user);

                if callback.send(channel).is_err() {
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
