use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};
use axum::async_trait;
use flume::Sender;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::manager::{Manager, ManagerChannel, ManagerReceiverTrait};
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use fydia_struct::{channel::ChannelId, user::UserId};
use parking_lot::Mutex;
use tokio::spawn;
use tokio::task::JoinError;

pub type TypingManager = Manager<TypingStruct>;

#[derive(Debug)]
pub enum TypingMessage {
    SetWebSocketManager(Arc<WebsocketManagerChannel>),
    SetTypingManager(Arc<TypingManagerChannel>),
    StartTyping(UserId, ChannelId, ServerId, Vec<User>),
    StopTyping(UserId, ChannelId, ServerId, Vec<User>),
    RemoveTask(UserId, ChannelId, ServerId, Vec<User>),
}

#[derive(Debug, Default)]
pub struct TypingStruct {
    wbsocketmanager: Option<Arc<WebsocketManagerChannel>>,
    selfmanager: Option<Arc<TypingManagerChannel>>,
    inner: TypingInner,
}

impl TypingStruct {
    pub fn set_websocketmanager(&mut self, websocket: Arc<WebsocketManagerChannel>) {
        self.wbsocketmanager = Some(websocket);
    }

    pub fn set_selfmanager(&mut self, selfmanager: Arc<TypingManagerChannel>) {
        self.selfmanager = Some(selfmanager);
    }
}

#[async_trait]
impl ManagerReceiverTrait for TypingStruct {
    type Message = TypingMessage;

    async fn on_receiver(&mut self, message: Self::Message) {
        match message {
            TypingMessage::SetWebSocketManager(wbmessage) => {
                self.set_websocketmanager(wbmessage);
            }
            TypingMessage::SetTypingManager(typingmanager) => {
                self.set_selfmanager(typingmanager);
            }
            TypingMessage::StartTyping(user, channelid, serverid, channel_user) => {
                if let (Some(wb), Some(typing)) = (&self.wbsocketmanager, &self.selfmanager) {
                    self.inner
                        .insert(
                            user,
                            channelid,
                            serverid,
                            channel_user,
                            wb.clone(),
                            typing.clone(),
                        )
                        .await
                }
            }
            TypingMessage::StopTyping(user, channelid, serverid, users_of_channel) => {
                if self.inner.stop_typing(&user, &channelid).is_ok() {
                    if let Some(wb) = &self.wbsocketmanager {
                        if self
                            .inner
                            .send_stop_typing(
                                serverid,
                                user,
                                channelid,
                                users_of_channel,
                                wb.clone(),
                            )
                            .await
                            .is_err()
                        {
                            error!("Error");
                        };
                    }
                }
            }

            TypingMessage::RemoveTask(user, channelid, serverid, users_of_channel) => {
                if self.inner.remove_task(&user, &channelid).is_ok() {
                    if let Some(wb) = &self.wbsocketmanager {
                        if self
                            .inner
                            .send_stop_typing(
                                serverid,
                                user,
                                channelid,
                                users_of_channel,
                                wb.clone(),
                            )
                            .await
                            .is_err()
                        {
                            error!("Error");
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TypingInner(Mutex<HashMap<ChannelId, Vec<UserTyping>>>);

impl TypingInner {
    pub fn channel_exists(&self, channelid: &ChannelId) -> bool {
        self.0.lock().get(channelid).is_some()
    }

    pub fn user_exists_in_channel(&self, channelid: &ChannelId, userid: &UserId) -> bool {
        if let Some(channel) = self.0.lock().get(channelid) {
            for i in channel {
                if &i.0 == userid {
                    println!("{}, {}", i.0.id, userid.id);
                    return true;
                }
            }
        }

        false
    }

    pub fn insert_channel(&self, channelid: ChannelId) {
        if !self.channel_exists(&channelid) {
            self.0.lock().insert(channelid, vec![]);
        }
    }

    pub async fn insert(
        &mut self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        channel_user: Vec<User>,
        websocket: Arc<WebsocketManagerChannel>,
        selfmanager: Arc<TypingManagerChannel>,
    ) {
        let mut task = Task::new(
            selfmanager,
            channel_user.clone(),
            userid.clone(),
            channelid.clone(),
            serverid.clone(),
        );

        if let Err(error) = task.spawn().await {
            error!(error);
            return;
        }

        self.insert_channel(channelid.clone());

        if self.user_exists_in_channel(&channelid, &userid) {
            error!("User already exists in channel");
            return;
        }

        if let Err(error) = self.stop_typing(&userid, &channelid) {
            error!(error);
        }

        if self
            .send_start_typing(
                serverid,
                userid.clone(),
                channelid.clone(),
                channel_user,
                websocket,
            )
            .await
            .is_err()
        {
            error!("Can't Send Message");
        }

        self.insert_new_user_typing(&channelid, UserTyping::new(userid, task));
    }

    pub fn insert_new_user_typing(&self, channelid: &ChannelId, usertyping: UserTyping) {
        if let Some(value) = self.0.lock().get_mut(&channelid) {
            value.push(usertyping);
        }
    }

    pub async fn send_start_typing(
        &self,
        serverid: ServerId,
        userid: UserId,
        channelid: ChannelId,
        channel_user: Vec<User>,
        websocket: Arc<WebsocketManagerChannel>,
    ) -> Result<(), ()> {
        websocket
            .send(
                Event::new(serverid, EventContent::StartTyping { userid, channelid }),
                channel_user,
            )
            .await
            .map_err(|_| ())
    }

    pub async fn send_stop_typing(
        &self,
        serverid: ServerId,
        userid: UserId,
        channelid: ChannelId,
        channel_user: Vec<User>,
        websocket: Arc<WebsocketManagerChannel>,
    ) -> Result<(), ()> {
        websocket
            .send(
                Event::new(serverid, EventContent::StopTyping { userid, channelid }),
                channel_user,
            )
            .await
            .map_err(|_| ())
    }

    pub fn remove_channel(&mut self, channelid: &ChannelId) {
        self.0.lock().remove(channelid);
    }

    pub fn get_index_of_user_of_channelid(
        &self,
        user: &UserId,
        channelid: &ChannelId,
    ) -> Option<usize> {
        let mut locked = self.0.lock();
        let uservec = locked.get_mut(channelid)?;
        for (n, usertyping) in uservec.iter().enumerate() {
            if &usertyping.0 == user {
                return Some(n);
            }
        }

        None
    }

    pub fn stop_typing(&mut self, user: &UserId, channelid: &ChannelId) -> Result<(), String> {
        self.remove_task(user, channelid)
            .map_err(|_| "No user exists in channel".to_string())
    }

    pub fn remove_task(&mut self, user: &UserId, channelid: &ChannelId) -> Result<(), ()> {
        if let Some(index) = self.get_index_of_user_of_channelid(user, channelid) {
            self.kill_task(channelid, index);

            if let Some(usertypings) = self.0.lock().get_mut(channelid) {
                usertypings.remove(index);
            }

            return Ok(());
        }

        Err(())
    }

    pub fn kill_task(&mut self, channelid: &ChannelId, index: usize) {
        if let Some(value) = self.0.lock().get_mut(channelid) {
            value[index].1.lock().kill();
        }
    }
}

impl Default for TypingInner {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

#[derive(Debug)]
pub struct UserTyping(UserId, Mutex<Task>);

impl UserTyping {
    pub fn new(userid: UserId, task: Task) -> Self {
        Self(userid, Mutex::new(task))
    }
}

#[derive(Debug, Clone)]
pub struct Task(Option<Arc<Sender<bool>>>, TaskValue);
#[derive(Clone, Debug)]
pub struct TaskValue(
    Arc<TypingManagerChannel>,
    Vec<User>,
    UserId,
    ChannelId,
    ServerId,
);

impl Task {
    pub fn new(
        typingsocketmanager: Arc<TypingManagerChannel>,
        user_vec: Vec<User>,
        executor: UserId,
        channelid: ChannelId,
        serverid: ServerId,
    ) -> Self {
        Self(
            None,
            TaskValue(typingsocketmanager, user_vec, executor, channelid, serverid),
        )
    }

    pub async fn spawn(&mut self) -> Result<(), String> {
        let task = self.clone();
        let (thread_sender, thread_receiver) = flume::bounded::<flume::Sender<bool>>(1);
        let _: Result<Result<(), String>, JoinError> = tokio::task::spawn(async move {
            let (sender, receiver) = flume::bounded::<bool>(1);
            // Will send the sender of thread for keep it alive
            thread_sender.send(sender).map_err(|f| f.to_string())?;
            let value = task.1;

            let instant = Instant::now();
            let need_to_kill = |instant: Instant| instant.elapsed().as_secs() == 10;
            spawn(async move {
                loop {
                    if need_to_kill(instant) {
                        if let Err(error) = value.0.remove_task(value.2, value.3, value.4, value.1)
                        {
                            error!(error);
                        }

                        return;
                    }
                    if let Ok(value) = receiver.recv_timeout(Duration::from_millis(10)) {
                        if value {
                            println!("Kill my self");
                            return;
                        }
                    }
                }
            });

            Ok(())
        })
        .await;

        let value = thread_receiver.recv().map_err(|f| f.to_string())?;
        self.0 = Some(Arc::new(value));

        Ok(())
    }

    pub fn kill(&mut self) {
        let clone_self = self.clone();
        if let Some(sender) = &clone_self.0 {
            if sender.send(true).is_err() {
                error!("Error");
            }

            self.0 = None;
        }
    }
}

pub type TypingManagerChannel = ManagerChannel<TypingMessage>;

pub trait TypingManagerChannelTrait {
    fn set_websocketmanager(&self, wbsocket: Arc<WebsocketManagerChannel>) -> Result<(), String>;
    fn set_selfmanager(&self, selfmanager: Arc<TypingManagerChannel>) -> Result<(), String>;
    fn start_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String>;
    fn stop_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String>;
    fn remove_task(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        users_of_channel: Vec<User>,
    ) -> Result<(), String>;
}

impl TypingManagerChannelTrait for TypingManagerChannel {
    fn set_websocketmanager(&self, wbsocket: Arc<WebsocketManagerChannel>) -> Result<(), String> {
        self.0
            .send(TypingMessage::SetWebSocketManager(wbsocket))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn set_selfmanager(&self, selfmanager: Arc<TypingManagerChannel>) -> Result<(), String> {
        self.0
            .send(TypingMessage::SetTypingManager(selfmanager))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn start_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String> {
        self.0
            .send(TypingMessage::StartTyping(
                userid,
                channelid,
                serverid,
                user_of_channel,
            ))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn stop_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String> {
        self.0
            .send(TypingMessage::StopTyping(
                userid,
                channelid,
                serverid,
                user_of_channel,
            ))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn remove_task(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        users_of_channel: Vec<User>,
    ) -> Result<(), String> {
        self.0
            .send(TypingMessage::RemoveTask(
                userid,
                channelid,
                serverid,
                users_of_channel,
            ))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }
}
