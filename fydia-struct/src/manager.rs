use std::{fmt::Debug, marker::PhantomData, process::exit};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

#[async_trait::async_trait]
pub trait ManagerReceiverTrait {
    type Message;
    async fn on_receiver(&mut self, message: Self::Message);
}

pub struct Manager<T: std::default::Default + Debug> {
    value: PhantomData<T>,
}

impl<T: std::default::Default + Debug + ManagerReceiverTrait + std::marker::Send> Manager<T> {
    pub async fn spawn() -> ManagerChannel<<T as ManagerReceiverTrait>::Message>
    where
        <T as ManagerReceiverTrait>::Message: Send + 'static,
    {
        let (ossender, receiver) =
            oneshot::channel::<UnboundedSender<<T as ManagerReceiverTrait>::Message>>();
        let error_exit = || {
            println!("Error on WBManager Init");
            exit(1);
        };
        tokio::spawn(async move {
            let mut websockets = T::default();
            let (sender, mut receiver) =
                tokio::sync::mpsc::unbounded_channel::<<T as ManagerReceiverTrait>::Message>();
            if ossender.send(sender.clone()).is_err() {
                error_exit();
            }

            while let Some(message) = receiver.recv().await {
                websockets.on_receiver(message).await
            }
        });
        if let Ok(sender) = receiver.await {
            ManagerChannel(sender)
        } else {
            error_exit()
        }
    }
}

pub struct ManagerChannel<T>(pub UnboundedSender<T>);
