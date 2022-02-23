//! This module related to manager

use std::sync::Arc;
use std::{fmt::Debug, marker::PhantomData, process::exit};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

/// Trait of Manager.
///
/// Manager is a thread with Channel to communicate with him.
#[async_trait::async_trait]
pub trait ManagerReceiverTrait {
    /// Type of Message
    type Message;

    /// Handler of a new Message
    async fn on_receiver(&mut self, message: Self::Message);
}

/// `Manager` struct with a generic value that implement `ManagerReceiverTrait`
#[derive(Debug)]
pub struct Manager<T: std::default::Default + Debug + ManagerReceiverTrait> {
    value: PhantomData<T>,
}

impl<T: std::default::Default + Debug + ManagerReceiverTrait + std::marker::Send> Manager<T> {
    /// Spawner of a Manager.
    /// This will start the manager.
    pub async fn spawn() -> ManagerChannel<<T as ManagerReceiverTrait>::Message>
    where
        <T as ManagerReceiverTrait>::Message: Send + 'static,
    {
        let (ossender, receiver) =
            oneshot::channel::<UnboundedSender<<T as ManagerReceiverTrait>::Message>>();
        let error_exit = || {
            println!("Error on Manager Init");
            exit(1);
        };
        tokio::spawn(async move {
            let mut value = T::default();
            let (sender, mut receiver) =
                tokio::sync::mpsc::unbounded_channel::<<T as ManagerReceiverTrait>::Message>();
            if ossender.send(sender.clone()).is_err() {
                error_exit();
            }

            while let Some(message) = receiver.recv().await {
                value.on_receiver(message).await
            }
        });
        if let Ok(sender) = receiver.await {
            ManagerChannel(Arc::new(sender))
        } else {
            error_exit()
        }
    }
}

/// ManagerChannel contains channel to communicate with Manager
#[derive(Debug, Clone)]
pub struct ManagerChannel<T>(pub Arc<UnboundedSender<T>>);
