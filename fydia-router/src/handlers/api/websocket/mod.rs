use async_channel::Receiver;
use async_channel::Sender;
use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_struct::channel::ChannelId;
use fydia_struct::event::EventContent;
use fydia_struct::messages::{Message, MessageType, SqlDate};
use fydia_struct::server::ServerId;
use fydia_struct::{
    event::Event,
    instance::{Instance, RsaData},
    user::User,
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender as OSSender;
pub mod messages;

pub type WbChannel = (Sender<ChannelMessage>, Receiver<ChannelMessage>);

#[derive(Debug)]
struct WbStruct(User, WbChannel, u32);
impl WbStruct {
    pub fn new(user: User, channel: WbChannel) -> Self {
        Self(
            User {
                id: user.id,
                name: user.name,
                ..Default::default()
            },
            channel,
            1,
        )
    }

    pub fn get_channel(&mut self) -> WbChannel {
        self.2 += 1;
        self.1.clone()
    }

    pub fn get_without_increment(&self) -> WbChannel {
        self.1.clone()
    }

    pub fn is_same_user(&self, user: &User) -> bool {
        self.0
            == User {
                id: user.id,
                name: user.name.clone(),
                ..Default::default()
            }
    }

    pub fn decrement_ref(&mut self) -> bool {
        self.2 -= 1;
        if self.2 == 0 {
            return true;
        }

        false
    }
}

impl Default for WebsocketInner {
    fn default() -> Self {
        Self {
            wb_channel: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Debug)]
pub struct WebsocketInner {
    wb_channel: Mutex<Vec<WbStruct>>,
}

impl WebsocketInner {
    pub fn new() -> Self {
        Default::default()
    }
    pub async fn get_or_insert_channels(&mut self, user: User) -> WbChannel {
        if let Some(channels) = self.get_channels_of_user(&user).await {
            channels
        } else {
            let channels = async_channel::unbounded::<ChannelMessage>();
            self.insert_channel(user, channels.clone()).await;
            channels
        }
    }
    pub async fn get_without_increment(&mut self, user: User) -> Option<WbChannel> {
        self.get_channels_of_user_without_increment(&user).await
    }
    pub async fn get_channels_of_user_without_increment(&self, user: &User) -> Option<WbChannel> {
        for i in self.wb_channel.lock().iter_mut() {
            if i.is_same_user(user) {
                return Some(i.get_without_increment());
            }
        }

        None
    }
    pub async fn get_channels_of_user(&self, user: &User) -> Option<WbChannel> {
        for i in self.wb_channel.lock().iter_mut() {
            if i.is_same_user(user) {
                return Some(i.get_channel());
            }
        }

        None
    }

    pub async fn insert_channel(&mut self, user: User, channel: WbChannel) {
        self.wb_channel.lock().push(WbStruct::new(user, channel));
    }

    pub fn remove(&mut self, index: usize) {
        self.wb_channel.lock().remove(index);
    }

    pub async fn decrement_and_remove_user(&mut self, user: User) {
        let mut user = user;
        user.password = None;
        let mut wblist = self.wb_channel.lock();
        for (n, i) in wblist.iter_mut().enumerate() {
            if i.is_same_user(&user) && i.decrement_ref() {
                wblist.remove(n);
                return;
            }
        }
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
    GetWithoutIncrement(User, OSSender<Option<WbChannel>>),
    GetActualOf(User, OSSender<WbChannel>),
    RemoveIfLast(User),
}
#[derive(Debug)]
pub struct WebsocketManager;

impl WebsocketManager {
    pub async fn spawn() -> WebsocketManagerChannel {
        let (sender, receiver) = async_channel::unbounded::<WbManagerMessage>();
        tokio::spawn(async move {
            let mut websockets = WebsocketInner::new();

            while let Ok(message) = receiver.recv().await {
                match message {
                    WbManagerMessage::GetActualOf(user, callback) => {
                        if callback
                            .send(websockets.get_or_insert_channels(user).await)
                            .is_err()
                        {
                            error!("Can't send");
                        };
                    }
                    WbManagerMessage::RemoveIfLast(user) => {
                        websockets.decrement_and_remove_user(user).await;
                    }
                    WbManagerMessage::GetWithoutIncrement(user, callback) => {
                        if callback
                            .send(
                                websockets
                                    .get_channels_of_user_without_increment(&user)
                                    .await,
                            )
                            .is_err()
                        {
                            error!("Can't send");
                        };
                    }
                }
                println!("{:?}", websockets);
            }
        });

        WebsocketManagerChannel(sender)
    }
}

#[derive(Clone, Debug)]
pub struct WebsocketManagerChannel(pub Sender<WbManagerMessage>);

impl WebsocketManagerChannel {
    async fn get_channels_of_user(&self, user: User) -> Result<WbChannel, String> {
        let (sender, receiver) = oneshot::channel::<WbChannel>();
        if let Err(e) = self
            .0
            .send(WbManagerMessage::GetActualOf(user, sender))
            .await
        {
            error!(e.to_string());
        }

        match receiver.await {
            Ok(e) => Ok(e),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn get_channels_of_user_without_increment(
        &self,
        user: User,
    ) -> Result<Option<WbChannel>, String> {
        let (sender, receiver) = oneshot::channel::<Option<WbChannel>>();
        if let Err(e) = self
            .0
            .send(WbManagerMessage::GetWithoutIncrement(user, sender))
            .await
        {
            error!(e.to_string());
        }

        match receiver.await {
            Ok(e) => Ok(e),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn close_connexion(&self, user: User) {
        if let Err(e) = self.0.send(WbManagerMessage::RemoveIfLast(user)).await {
            error!(e.to_string());
        }
    }

    pub async fn send(
        &self,
        msg: Event,
        user: Vec<User>,
        _keys: Option<&RsaData>,
        _origin: Option<Instance>,
    ) -> Result<(), ()> {
        for mut i in user {
            i.drop_password();
            if let Ok(wbstruct) = self.get_channels_of_user_without_increment(i).await {
                if let Some(wbstruct) = wbstruct {
                    let msg = ChannelMessage::Message(Box::new(msg.clone()));
                    if let Err(e) = wbstruct.0.send(msg).await {
                        error!(e.to_string());
                    }
                } else {
                    return Ok(());
                }
            } else {
                return Err(());
            }
        }

        Ok(())
    }
}

pub async fn test_message(
    Extension(websocket_manager_channel): Extension<Arc<WebsocketManagerChannel>>,
) -> impl IntoResponse {
    let instant = Instant::now();
    let getted_websocket = websocket_manager_channel
        .get_channels_of_user(User::default())
        .await
        .unwrap();
    if let Err(e) = getted_websocket
        .0
        .send(ChannelMessage::Message(Box::new(Event::new(
            ServerId::new(String::new()),
            EventContent::Message {
                content: Message::new(
                    String::new(),
                    MessageType::TEXT,
                    false,
                    SqlDate::now(),
                    User::default(),
                    ChannelId::default(),
                ),
            },
        ))))
        .await
    {
        println!("{}", e.to_string())
    };

    format!(
        "{}µs => {:?}",
        instant.elapsed().as_micros(),
        getted_websocket
    )
}
