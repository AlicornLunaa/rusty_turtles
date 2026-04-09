use tokio::{sync::mpsc, task::JoinHandle};

/// This module contains the server controller for incoming requests
pub enum ServerMessage {
    Ping,
}

pub struct Gateway {
    join_handle: JoinHandle<()>,
    sender: mpsc::Sender<ServerMessage> // Used for cloning
}

impl Gateway {
    pub fn new() -> Self {
        // Start a MPSC channel to handle incoming requests
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn gateway thread to handle incoming requests
        let join_handle = tokio::spawn(async move {
            println!("Starting gateway thread");

            while let Some(message) = rx.recv().await {
                match message {
                    ServerMessage::Ping => {
                        println!("Ping received");
                    },
                }
            }

            println!("Gateway thread ended");
        });

        Gateway {
            join_handle,
            sender: tx
        }
    }

    pub fn get_sender(&self) -> mpsc::Sender<ServerMessage> {
        self.sender.clone()
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}