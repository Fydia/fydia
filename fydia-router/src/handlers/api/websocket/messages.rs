use crate::handlers::api::websocket::{accept, requested, ChannelMessage, Websockets};
use futures::prelude::*;
use fydia_sql::impls::token::SqlToken;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::querystring::QsToken;
use fydia_struct::user::Token;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::hyper::upgrade::OnUpgrade;
use gotham::hyper::{Body, HeaderMap, Response, StatusCode};
use gotham::state::{FromState, State};
use logger::info;
use serde::Serialize;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::Error;

const INDEX: &str = include_str!("../../../../index.html");

pub async fn ws_handler(mut state: State) -> HandlerResult {
    let headers = HeaderMap::take_from(&mut state);
    let on_upgrade = OnUpgrade::try_take_from(&mut state);
    let database = &SqlPool::borrow_from(&state).get_pool();
    let token = Token(
        QsToken::borrow_from(&state)
            .token
            .clone()
            .unwrap_or_default(),
    );
    let mut res = Response::new(Body::from(INDEX));
    info!(format!("Get one connection : {}", token.0));
    if let Some(user) = token.get_user(database).await {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<ChannelMessage>();
        match on_upgrade {
            Some(on_upgrade) if requested(&headers) => {
                let (response, ws) = match accept(&headers, on_upgrade) {
                    Ok(res) => res,
                    Err(_) => {
                        let res = create_response(
                            &state,
                            StatusCode::BAD_REQUEST,
                            mime::TEXT_PLAIN_UTF_8,
                            Body::empty(),
                        );
                        return Ok((state, res));
                    }
                };

                Websockets::borrow_mut_from(&mut state)
                    .channel
                    .lock()
                    .await
                    .insert(sender.clone(), user);
                Websockets::borrow_mut_from(&mut state)
                    .channel
                    .lock()
                    .await
                    .remove_unvalid_channel();
                println!(
                    "{:#?}",
                    Websockets::borrow_mut_from(&mut state).channel.lock().await
                );

                tokio::spawn(async move {
                    match ws.await {
                        Ok(ws) => connected(ws, sender, receiver).await,
                        Err(err) => {
                            eprintln!("websocket init error: {}", err);
                            Err(())
                        }
                    }
                });

                res = response;
            }
            _ => res = Response::new(Body::from(INDEX)),
        }
    }

    Ok((state, res))
}

async fn connected<S>(
    stream: S,
    sendchannel: tokio::sync::mpsc::UnboundedSender<ChannelMessage>,
    mut receiverchannel: tokio::sync::mpsc::UnboundedReceiver<ChannelMessage>,
) -> Result<(), ()>
where
    S: Stream<Item = Result<Message, Error>>
        + Sink<Message, Error = Error>
        + std::marker::Send
        + 'static,
{
    let (mut sink, mut stream) = stream.split();

    let websocket_stream = sendchannel.clone();
    let task = tokio::spawn(async move {
        while let Ok(Some(message)) = stream
            .next()
            .await
            .transpose()
            .map_err(|error| println!("Websocket receive error: {}", error))
        {
            if websocket_stream
                .send(ChannelMessage::WebsocketMessage(message))
                .is_err()
            {
                error!("Can't send message");
            }
        }
    });

    while let Some(msg) = receiverchannel.recv().await {
        match msg {
            ChannelMessage::WebsocketMessage(msg) => match sink.send(msg).await {
                Ok(()) => {}
                Err(Error::ConnectionClosed) => {}
                Err(error) => {
                    println!("{:?}", error);
                    return Err(());
                }
            },
            ChannelMessage::Kill => {
                if sink.close().await.is_err() {
                    error!("Can't close channel");
                };
                break;
            }
            ChannelMessage::Message(msg) => {
                if let Ok(msg) = to_websocketmessage(&msg) {
                    match sink.send(msg).await {
                        Ok(()) => {}
                        Err(Error::ConnectionClosed) => {}
                        Err(error) => {
                            println!("{:?}", error);
                            return Err(());
                        }
                    }
                }
            }
        }
    }

    task.abort();

    Ok(())
}

pub fn to_websocketmessage<T>(msg: &T) -> Result<Message, ()>
where
    T: Serialize,
{
    match serde_json::to_string(msg) {
        Ok(json) => Ok(Message::from(json)),
        _ => Err(()),
    }
}
