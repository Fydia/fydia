use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::handlers::api::manager::websockets::manager::{
    WbManagerChannelTrait, WebsocketManagerChannel,
};
use axum::async_trait;
use flume::Sender;
use fydia_sql::impls::channel::{SqlChannel, SqlChannelId};
use fydia_sql::impls::server::SqlMember;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::event::{Event, EventContent};
use fydia_struct::manager::{Manager, ManagerChannel, ManagerReceiverTrait};
use fydia_struct::server::ServerId;
use fydia_struct::{channel::ChannelId, user::UserId};
use parking_lot::RwLock;

pub type TypingManager = Manager<TypingStruct>;

#[derive(Debug)]
pub enum TypingMessage {
    SetWebSocketManager(Arc<WebsocketManagerChannel>),
    SetTypingManager(Arc<TypingManagerChannel>),
    SetDatabase(DbConnection),
    StartTyping(UserId, ChannelId, ServerId),
    StopTyping(UserId, ChannelId, ServerId),
}

#[derive(Debug, Default)]
pub struct TypingStruct {
    wbsocketmanager: Option<Arc<WebsocketManagerChannel>>,
    selfmanager: Option<Arc<TypingManagerChannel>>,
    database: Option<DbConnection>,
    inner: TypingInner,
}

impl TypingStruct {
    pub fn set_websocketmanager(&mut self, websocket: Arc<WebsocketManagerChannel>) {
        self.wbsocketmanager = Some(websocket);
    }

    pub fn set_selfmanager(&mut self, selfmanager: Arc<TypingManagerChannel>) {
        self.selfmanager = Some(selfmanager);
    }

    pub fn set_database(&mut self, database: DbConnection) {
        self.database = Some(database);
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
            TypingMessage::SetDatabase(databasemanager) => {
                self.set_database(databasemanager);
            }
            TypingMessage::StartTyping(user, channelid, serverid) => {
                if let (Some(wb), Some(typing), Some(database)) =
                    (&self.wbsocketmanager, &self.selfmanager, &self.database)
                {
                    if let Err(error) = self
                        .inner
                        .start_typing(database, wb, typing, channelid, user, serverid)
                        .await
                    {
                        error!("{error}");
                    };
                }
            }
            TypingMessage::StopTyping(user, channelid, serverid) => {
                if let (Some(wb), Some(database)) = (&self.wbsocketmanager, &self.database) {
                    if let Err(error) = self
                        .inner
                        .stop_typing(database, wb, channelid, user, serverid)
                        .await
                    {
                        error!("{error}");
                    };
                };
            }
        }
    }
}

#[derive(Debug)]
pub struct TypingInner(RwLock<HashMap<ChannelId, HashMap<UserId, Task>>>);

impl TypingInner {
    /// Renew a typing task
    ///
    /// # Errors
    /// Return an error if :
    /// * `ChannelId` isn't in hashmap
    pub async fn renew_typing(
        &mut self,
        typingmanager: &Arc<TypingManagerChannel>,
        channelid: ChannelId,
        userid: UserId,
        serverid: ServerId,
    ) -> Result<(), String> {
        let mut inner = self.0.write();
        let usertypings = inner.get_mut(&channelid).ok_or("No Channel in HashMap")?;
        let mut task = Task::new(typingmanager, &userid, &channelid, &serverid);

        task.spawn();
        usertypings.insert(userid, task);

        Ok(())
    }
    /// Stop Typing
    ///
    /// # Errors
    /// Return an error if :
    /// * [`self::renew_typing`] return an error
    pub async fn start_typing(
        &mut self,
        database: &DbConnection,
        websocket: &Arc<WebsocketManagerChannel>,
        typingmanager: &Arc<TypingManagerChannel>,
        channelid: ChannelId,
        userid: UserId,
        serverid: ServerId,
    ) -> Result<(), String> {
        let is_exists = self.user_exists_in_channel(&channelid, &userid);
        if is_exists {
            return self
                .renew_typing(typingmanager, channelid, userid, serverid)
                .await;
        }

        let task = Task::new(typingmanager, &userid, &channelid, &serverid);

        self.insert_user(&channelid, &userid, task);

        send_websocket_message(
            EventContent::StartTyping {
                userid,
                channelid: channelid.clone(),
            },
            serverid,
            channelid,
            websocket,
            database,
        );

        Ok(())
    }

    pub fn insert_channel(&mut self, channelid: &ChannelId) {
        let mut inner = self.0.write();
        if !inner.contains_key(channelid) {
            inner.insert(channelid.clone(), HashMap::new());
        }
    }

    pub fn insert_user(&mut self, channelid: &ChannelId, userid: &UserId, mut task: Task) {
        self.insert_channel(channelid);
        println!("Channel Insert");
        let mut inner = self.0.write();
        if let Some(channel) = inner.get_mut(channelid) {
            if !channel.contains_key(userid) {
                task.spawn();
                channel.insert(userid.clone(), task);
            }
        }
    }

    pub fn user_exists_in_channel(&self, channelid: &ChannelId, userid: &UserId) -> bool {
        if let Some(channel) = self.0.read().get(channelid) {
            if channel.contains_key(userid) {
                return true;
            }
        }

        false
    }

    /// Stop Typing
    ///
    /// # Errors
    /// Return an error if:
    /// * `ChannelId` isn't in hashmap
    /// * `UserId` isn't in hashmap
    pub async fn stop_typing(
        &mut self,
        database: &DbConnection,
        websocket: &Arc<WebsocketManagerChannel>,
        channelid: ChannelId,
        userid: UserId,
        serverid: ServerId,
    ) -> Result<(), String> {
        let mut inner = self.0.write();

        let users = inner
            .get_mut(&channelid)
            .ok_or_else(|| "No ChannelId in HashMap".to_string())?;

        let user_task = users
            .get_mut(&userid)
            .ok_or_else(|| "No User in HashMap".to_string())?;

        user_task.kill();

        users.remove(&userid);

        send_websocket_message(
            EventContent::StopTyping {
                userid,
                channelid: channelid.clone(),
            },
            serverid,
            channelid,
            websocket,
            database,
        );

        Ok(())
    }
}

impl Default for TypingInner {
    fn default() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
}

fn send_websocket_message(
    event: EventContent,
    serverid: ServerId,
    channelid: ChannelId,
    websocket: &Arc<WebsocketManagerChannel>,
    database: &DbConnection,
) {
    let database = database.clone();
    let websocket = websocket.clone();
    tokio::task::spawn(async move {
        let users = channelid
            .get_channel(&database)
            .await?
            .get_user_of_channel(&database)
            .await?
            .to_userinfo(&database)
            .await?;

        websocket
            .send(&Event::new(serverid.clone(), event), &users)
            .await
    });
}

#[derive(Debug, Clone)]
pub struct Task(Option<Arc<Sender<bool>>>, TaskValue); // Add Mutex
#[derive(Clone, Debug)]
pub struct TaskValue(Arc<TypingManagerChannel>, UserId, ChannelId, ServerId);

impl Task {
    pub fn new(
        typingsocketmanager: &Arc<TypingManagerChannel>,
        executor: &UserId,
        channelid: &ChannelId,
        serverid: &ServerId,
    ) -> Self {
        Self(
            None,
            TaskValue(
                typingsocketmanager.clone(),
                executor.clone(),
                channelid.clone(),
                serverid.clone(),
            ),
        )
    }

    pub fn spawn(&mut self) {
        let task = self.clone();
        let (sender, receiver) = flume::bounded::<bool>(1);
        let instant = Instant::now();
        let value = task.1;
        tokio::task::spawn(async move {
            loop {
                if instant.elapsed().as_secs() == 10 {
                    if let Err(error) = value.0.stop_typing(value.1, value.2, value.3) {
                        error!("{error}");
                    }

                    return;
                }
                if let Ok(value) = receiver.recv_timeout(Duration::from_millis(10)) {
                    if value {
                        return;
                    }
                }
            }
        });
        self.0 = Some(Arc::new(sender));
    }

    pub fn kill(&mut self) {
        if let Some(sender) = &self.0 {
            sender.send(true).unwrap_or(());

            self.0 = None;
        }
    }
}

pub type TypingManagerChannel = ManagerChannel<TypingMessage>;

pub trait TypingManagerChannelTrait {
    /// Set websocket manager
    ///
    /// # Errors
    /// Return an error if:
    /// * typing manager is unreachable
    fn set_websocketmanager(&self, wbsocket: &Arc<WebsocketManagerChannel>) -> Result<(), String>;

    /// Set a reference to self
    ///
    /// # Errors
    /// Return an error if:
    /// * typing manager is unreachable
    fn set_selfmanager(&self, selfmanager: &Arc<TypingManagerChannel>) -> Result<(), String>;

    /// Set database
    ///
    /// # Errors
    /// Return an error if:
    /// * typing manager is unreachable
    fn set_database(&self, dbconnection: &DbConnection) -> Result<(), String>;

    /// Start typing task
    ///
    /// # Errors
    /// Return an error if:
    /// * typing manager is unreachable
    fn start_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
    ) -> Result<(), String>;

    /// Stop typing task
    ///
    /// # Errors
    /// Return an error if:
    /// * typing manager is unreachable
    fn stop_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
    ) -> Result<(), String>;
}

impl TypingManagerChannelTrait for TypingManagerChannel {
    fn set_websocketmanager(&self, wbsocket: &Arc<WebsocketManagerChannel>) -> Result<(), String> {
        self.0
            .send(TypingMessage::SetWebSocketManager(wbsocket.clone()))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn set_selfmanager(&self, selfmanager: &Arc<TypingManagerChannel>) -> Result<(), String> {
        self.0
            .send(TypingMessage::SetTypingManager(selfmanager.clone()))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn set_database(&self, dbconnection: &DbConnection) -> Result<(), String> {
        self.0
            .send(TypingMessage::SetDatabase(dbconnection.clone()))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn start_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
    ) -> Result<(), String> {
        self.0
            .send(TypingMessage::StartTyping(userid, channelid, serverid))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }

    fn stop_typing(
        &self,
        userid: UserId,
        channelid: ChannelId,
        serverid: ServerId,
    ) -> Result<(), String> {
        self.0
            .send(TypingMessage::StopTyping(userid, channelid, serverid))
            .map(|_| ())
            .map_err(|f| f.to_string())
    }
}
