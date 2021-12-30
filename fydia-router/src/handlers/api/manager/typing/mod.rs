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
                        .await;
                }
            }
            TypingMessage::StopTyping(user, channelid, serverid, channel_of_user) => {
                self.inner.stop_typing(&user, &channelid).await;
                if let Some(wb) = &self.wbsocketmanager {
                    if wb
                        .send(
                            Event::new(
                                serverid,
                                EventContent::StopTyping {
                                    userid: user,
                                    channelid,
                                },
                            ),
                            channel_of_user,
                            None,
                            None,
                        )
                        .await
                        .is_err()
                    {
                        error!("Error");
                    }
                }
            }

            TypingMessage::RemoveTask(user, channelid, serverid, users_of_channel) => {
                self.inner.remove_task(&user, &channelid);
                if let Some(wb) = &self.wbsocketmanager {
                    if wb
                        .send(
                            Event::new(
                                serverid,
                                EventContent::StopTyping {
                                    userid: user,
                                    channelid,
                                },
                            ),
                            users_of_channel,
                            None,
                            None,
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

#[derive(Debug)]
pub struct TypingInner(Mutex<HashMap<ChannelId, Vec<UserTyping>>>);

impl TypingInner {
    pub async fn insert(
        &mut self,
        user: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        channel_user: Vec<User>,
        websocket: Arc<WebsocketManagerChannel>,
        selfmanager: Arc<TypingManagerChannel>,
    ) {
        self.stop_typing(&user, &channelid).await;
        warn!("Will insert a new user");
        self.insert_user_task(
            user,
            channelid,
            serverid,
            channel_user,
            websocket,
            selfmanager,
        )
        .await;
    }

    async fn insert_user_task(
        &mut self,
        user: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        channel_user: Vec<User>,
        websocket: Arc<WebsocketManagerChannel>,
        selfmanager: Arc<TypingManagerChannel>,
    ) {
        let mut task = Task::new(
            selfmanager,
            channel_user.clone(),
            user.clone(),
            channelid.clone(),
            serverid.clone(),
        );
        task.spawn().await;
        self.0
            .lock()
            .insert(channelid.clone(), vec![UserTyping::new(user.clone(), task)]);

        if websocket
            .send(
                Event::new(
                    serverid,
                    EventContent::StartTyping {
                        userid: user,
                        channelid,
                    },
                ),
                channel_user,
                None,
                None,
            )
            .await.is_err() {
            error!("Can't insert task");
        };
    }

    pub async fn remove_channel(&mut self, channelid: &ChannelId) {
        self.0.lock().remove(channelid);
    }

    pub fn get_index_of_user_of_channelid(
        &self,
        user: &UserId,
        channelid: &ChannelId,
    ) -> Option<usize> {
        let mut locked = self.0.lock();
        if let Some(uservec) = locked.get_mut(channelid) {
            for (n, usertyping) in uservec.iter().enumerate() {
                if &usertyping.0.lock().0 == user {
                    return Some(n);
                }
            }
        }

        None
    }

    pub async fn stop_typing(&mut self, user: &UserId, channelid: &ChannelId) {
        if let Some(n) = self.get_index_of_user_of_channelid(user, channelid) {
            self.kill_task(channelid, n);
            self.remove_task_with_index(channelid, n);
        }
    }

    pub fn remove_task(&mut self, user: &UserId, channelid: &ChannelId) {
        if let Some(index) = self.get_index_of_user_of_channelid(user, channelid) {
            self.remove_task_with_index(channelid, index);
        }
    }

    pub fn remove_task_with_index(&mut self, channelid: &ChannelId, index: usize) {
        if let Some(value) = self.0.lock().get_mut(channelid) {
            value.remove(index);
        }
    }

    pub fn kill_task(&mut self, channelid: &ChannelId, index: usize) {
        if let Some(value) = self.0.lock().get_mut(channelid) {
            value[index].0.lock().1.kill();
        }
    }
}

impl Default for TypingInner {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

#[derive(Debug)]
pub struct UserTyping(Mutex<(UserId, Task)>);

impl UserTyping {
    pub fn new(userid: UserId, task: Task) -> Self {
        Self(Mutex::new((userid, task)))
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

    pub async fn spawn(&mut self) {
        let task = self.clone();
        let (thread_sender, thread_receiver) = flume::bounded::<flume::Sender<bool>>(1);
        let _ = tokio::task::spawn(async move {
            let instant = Instant::now();
            let (sender, receiver) = flume::bounded::<bool>(1);
            if let Err(error) = thread_sender.send(sender) {
                error!(error.to_string());
                return;
            }
            let value = task.1;
            spawn(async move {
                loop {
                    if receiver.recv_timeout(Duration::from_micros(10)).is_ok() {
                        return;
                    }
                    if instant.elapsed().as_secs() == 10 {
                        if let Err(error) = value.0.remove_task(value.2, value.3, value.4, value.1)
                        {
                            error!(error);
                        }

                        return;
                    }
                }
            });
        })
        .await;
        if let Ok(value) = thread_receiver.recv() {
            self.0 = Some(Arc::new(value));
        } else {
            panic!("AHAHAHHAHAHA");
        }
    }

    pub fn kill(&mut self) {
        let clone_self = self.clone();
        if let Some(sender) = &clone_self.0 {
            if sender.send(true).is_err() {
                error!("Error");
            }
            warn!("Task killed");
            self.0 = None;
        } else {
            warn!("No task");
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
        if let Err(error) = self.0.send(TypingMessage::SetWebSocketManager(wbsocket)) {
            return Err(error.to_string());
        }

        Ok(())
    }

    fn set_selfmanager(&self, selfmanager: Arc<TypingManagerChannel>) -> Result<(), String> {
        if let Err(error) = self.0.send(TypingMessage::SetTypingManager(selfmanager)) {
            return Err(error.to_string());
        }

        Ok(())
    }

    fn start_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String> {
        if let Err(error) = self.0.send(TypingMessage::StartTyping(
            userid,
            channelid,
            serverid,
            user_of_channel,
        )) {
            return Err(error.to_string());
        }

        Ok(())
    }

    fn stop_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        user_of_channel: Vec<User>,
    ) -> Result<(), String> {
        if let Err(error) = self.0.send(TypingMessage::StopTyping(
            userid,
            channelid,
            serverid,
            user_of_channel,
        )) {
            return Err(error.to_string());
        }

        Ok(())
    }

    fn remove_task(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
        users_of_channel: Vec<User>,
    ) -> Result<(), String> {
        if let Err(error) = self.0.send(TypingMessage::RemoveTask(
            userid,
            channelid,
            serverid,
            users_of_channel,
        )) {
            return Err(error.to_string());
        }

        Ok(())
    }
}
