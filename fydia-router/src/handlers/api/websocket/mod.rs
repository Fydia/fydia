use futures::Future;
use fydia_dispatcher::message::send::encrypt_message;
use fydia_struct::event::Event;
use fydia_struct::instance::{Instance, RsaData};
use fydia_struct::user::User;
use gotham::hyper::header::{
    HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE,
};
use gotham::hyper::upgrade::{OnUpgrade, Upgraded};
use gotham::hyper::{Body, HeaderMap, Response, StatusCode};
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub mod messages;

#[derive(StateData, Clone, Debug)]
pub struct Websockets {
    pub channel: Arc<Mutex<WbList>>,
}

impl Default for Websockets {
    fn default() -> Self {
        Self::new()
    }
}

impl Websockets {
    pub fn new() -> Self {
        Self {
            channel: Arc::new(Mutex::new(WbList::new())),
        }
    }

    pub async fn send(
        &mut self,
        msg: &Event,
        user: Vec<User>,
        keys: Option<&RsaData>,
        origin: Option<Instance>,
    ) {
        let mut e = self.channel.lock().await;
        e.remove_unvalid_channel();
        e.0.par_iter().for_each(|i| {
            if user.contains(&i.user) {
                if i.user.instance.domain == "localhost" {
                    i.channel
                        .send(ChannelMessage::Message(Box::new(msg.clone())))
                        .unwrap();
                } else if let Some(rsa) = keys {
                    let encrypt_message = encrypt_message(
                        rsa,
                        i.user.instance.get_public_key().unwrap(),
                        msg.clone(),
                    );
                    //send_message(rsa, origin,  i.user.instance.get_public_key().unwrap(), message, instances);
                }
            }
        });
    }
}

unsafe impl Send for Websockets {}
impl RefUnwindSafe for Websockets {}

#[derive(Debug, Clone)]
pub struct WbList(pub Vec<WbUser>);

impl WbList {
    pub fn new() -> Self {
        Self { 0: Vec::new() }
    }
    pub fn insert(&mut self, channel: UnboundedSender<ChannelMessage>, user: User) {
        self.0.push(WbUser::new(channel, user));
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
    pub fn get(&self) -> Vec<WbUser> {
        self.0.clone()
    }
    pub fn remove_unvalid_channel(&mut self) {
        if self.0.is_empty() {
            return;
        }
        for (index, wbuser) in self.0.clone().iter().enumerate() {
            if wbuser.channel.is_closed() {
                self.0.remove(index);
            };
        }
    }
}

impl Default for WbList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WbUser {
    pub channel: UnboundedSender<ChannelMessage>,
    pub user: User,
}

impl WbUser {
    pub fn new(channel: UnboundedSender<ChannelMessage>, user: User) -> Self {
        Self { channel, user }
    }
}

#[derive(Debug, Clone)]
pub enum ChannelMessage {
    WebsocketMessage(Message),
    Message(Box<Event>),
    Kill,
}

const PROTO_WEBSOCKET: &str = "websocket";

/// Check if a WebSocket upgrade was requested.
pub fn requested(headers: &HeaderMap) -> bool {
    headers.get(UPGRADE) == Some(&HeaderValue::from_static(PROTO_WEBSOCKET))
}

/// Accept a WebSocket upgrade request.
///
/// Returns HTTP response, and a future that eventually resolves
/// into websocket object.
pub fn accept(
    headers: &HeaderMap,
    on_upgrade: OnUpgrade,
) -> Result<
    (
        Response<Body>,
        impl Future<Output = Result<WebSocketStream<Upgraded>, gotham::hyper::Error>>,
    ),
    (),
> {
    let res = response(headers)?;
    let ws = async move {
        let upgraded = on_upgrade.await?;
        Ok(WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await)
    };

    Ok((res, ws))
}

fn response(headers: &HeaderMap) -> Result<Response<Body>, ()> {
    let key = headers.get(SEC_WEBSOCKET_KEY).ok_or(())?;

    Ok(Response::builder()
        .header(UPGRADE, PROTO_WEBSOCKET)
        .header(CONNECTION, "upgrade")
        .header(SEC_WEBSOCKET_ACCEPT, accept_key(key.as_bytes()))
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .body(Body::empty())
        .unwrap())
}

fn accept_key(key: &[u8]) -> String {
    const WS_GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut sha1 = sha1::Sha1::default();
    sha1.update(key);
    sha1.update(WS_GUID);
    base64::encode(&sha1.digest().bytes())
}
