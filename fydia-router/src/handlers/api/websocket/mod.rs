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
use std::time::Instant;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender as Sender};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender as OSSender;

pub mod messages;

#[derive(Debug, Clone)]
pub struct WebsocketInner {
    wb_channel: Vec<WbStruct>,
}

type WbStruct = (User, Vec<Sender<ChannelMessage>>);

impl Default for WebsocketInner {
    fn default() -> Self {
        Self {
            wb_channel: Vec::new(),
        }
    }
}

impl WebsocketInner {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn get_channels_of_user(&self, user: User) -> Option<Vec<Sender<ChannelMessage>>> {
        let mut user = user;
        user.password = None;
        for i in self.wb_channel.iter() {
            if i.0 == user {
                return Some(i.1.clone());
            }
        }

        None
    }

    pub async fn insert_channel(&mut self, user: User, channel: Sender<ChannelMessage>) {
        let mut user = user;

        user.password = None;

        self.wb_channel.iter_mut().for_each(|e| {
            if e.0 == user {
                e.1.push(channel.clone());
                return;
            }
        });

        self.wb_channel.push((user, vec![channel]));
    }

    pub async fn remove_channel_of_user(&mut self, user: User, sender: &Sender<ChannelMessage>) {
        let mut user = user;
        user.password = None;

        let mut index = None;
        for i in self.wb_channel.iter_mut() {
            if i.0 == user {
                for i in i.1.iter().enumerate() {
                    if i.1.same_channel(sender) {
                        index = Some(i.0);
                        break;
                    }
                }
                break;
            }
        }

        if let Some(i) = index {
            self.wb_channel.remove(i);
        };
    }
}

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    WebsocketMessage(axum::extract::ws::Message),
    Message(Event),
    Kill,
}

#[derive(Debug)]
pub enum WbManagerMessage {
    Add(User, Sender<ChannelMessage>),
    Get(OSSender<Vec<WbStruct>>),
    GetOfUser(User, OSSender<Option<Vec<Sender<ChannelMessage>>>>),
    Remove(User, Sender<ChannelMessage>),
}
#[derive(Debug)]
pub struct WebsocketManager;

impl WebsocketManager {
    pub async fn spawn() -> WebsocketManagerChannel {
        let (sender, mut receiver) = unbounded_channel::<WbManagerMessage>();
        tokio::spawn(async move {
            let mut websockets = WebsocketInner::new();

            while let Some(message) = receiver.recv().await {
                match message {
                    WbManagerMessage::Add(user, channel) => {
                        websockets.insert_channel(user, channel).await;
                        println!("Add")
                    }
                    WbManagerMessage::Remove(user, sender) => {
                        websockets.remove_channel_of_user(user, &sender).await
                    }
                    WbManagerMessage::Get(oneshot) => {
                        if oneshot.send(websockets.wb_channel.clone()).is_err() {
                            error!("Can't send");
                        };
                    }
                    WbManagerMessage::GetOfUser(user, channel) => {
                        if channel
                            .send(websockets.get_channels_of_user(user).await)
                            .is_err()
                        {
                            error!("Can't send");
                        };
                    }
                }
            }
        });

        WebsocketManagerChannel(sender)
    }
}

#[derive(Clone, Debug)]
pub struct WebsocketManagerChannel(pub Sender<WbManagerMessage>);

impl WebsocketManagerChannel {
    async fn _get(&self) -> Result<Vec<WbStruct>, tokio::sync::oneshot::error::RecvError> {
        let (sender, receiver) = oneshot::channel::<Vec<WbStruct>>();
        if let Err(e) = self.0.send(WbManagerMessage::Get(sender)) {
            error!(e.to_string());
        };

        receiver.await
    }
    pub fn insert_channel(&self, user: User, channel: Sender<ChannelMessage>) {
        if let Err(e) = self.0.send(WbManagerMessage::Add(user, channel)) {
            error!(e.to_string());
        }
    }
    pub fn remove_channel_of_user(&self, user: User, sender: &Sender<ChannelMessage>) {
        if let Err(e) = self.0.send(WbManagerMessage::Remove(user, sender.clone())) {
            error!(e.to_string());
        }
    }
    pub async fn get_channels_of_user(&self, user: User) -> Option<Vec<Sender<ChannelMessage>>> {
        let (sender, receiver) = oneshot::channel::<Option<Vec<Sender<ChannelMessage>>>>();
        if let Err(e) = self.0.send(WbManagerMessage::GetOfUser(user, sender)) {
            error!(e.to_string());
        }

        if let Ok(some) = receiver.await {
            some
        } else {
            None
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
            if let Some(wbstructs) = self.get_channels_of_user(i).await {
                for wbstruct in wbstructs {
                    let msg = ChannelMessage::Message(msg.clone());
                    if let Err(e) = wbstruct.send(msg) {
                        error!(e.to_string());
                    }
                }
            }
        }

        Ok(())
    }
}

pub async fn test_message(
    Extension(websocket_manager_channel): Extension<WebsocketManagerChannel>,
) -> impl IntoResponse {
    let instant = Instant::now();
    let getted_websocket = websocket_manager_channel
        .get_channels_of_user(User::default())
        .await;
    getted_websocket.iter().for_each(|e| {
        e.iter().for_each(|e| {
            if let Err(e) = e.send(ChannelMessage::Message(Event::new(
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
            ))) {
                println!("{}", e.to_string())
            };
        })
    });
    format!(
        "{}µs => {:?}",
        instant.elapsed().as_micros(),
        getted_websocket
    )
}
