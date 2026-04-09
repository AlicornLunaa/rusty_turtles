use std::sync::Arc;

use serde_json::{Value, json};
use tokio::{sync::{Mutex, mpsc, oneshot}, task::JoinHandle};

use crate::managers::{block_manager::BlockManager, turtle_manager::{self, TurtleManager}};

/// This module contains the server controller for incoming requests
pub enum ServerAction {
    Ping,
    SetupGPS,
    StopGPS,
}

pub enum ServerMessage {
    Procedure{ client_id: u64, action: ServerAction, tx: oneshot::Sender<Result<Value, String>>}, // The caller expects a response
    Oneshot{ client_id: u64, action: ServerAction }, // An action fired off and forgot about
}

pub struct Gateway {
    join_handle: JoinHandle<()>,
    sender: mpsc::Sender<ServerMessage>, // Used for cloning
    turtle_manager: Arc<Mutex<TurtleManager>>,
    block_manager: Arc<Mutex<BlockManager>>,
}

impl Gateway {
    async fn start_gps_procedure(){
        // This procedure finds 4 turtles which are not in use, tells them to move out into a constellation, then host GPS
        // for another turtle to locate itself for bootstrapping
    }

    pub fn new(turtle_manager: Arc<Mutex<TurtleManager>>, block_manager: Arc<Mutex<BlockManager>>) -> Self {
        // Start a MPSC channel to handle incoming requests
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn gateway thread to handle incoming requests
        let join_handle = tokio::spawn(async move {
            println!("Starting gateway thread");

            while let Some(message) = rx.recv().await {
                match message {
                    ServerMessage::Oneshot { client_id, action } => {
                        match action {
                            ServerAction::Ping => {
                                println!("Ping received");
                            },
                            _ => {
                                println!("Unknown oneshot action received");
                            }
                        }
                    },
                    ServerMessage::Procedure { client_id, action, tx } => {
                        match action {
                            ServerAction::Ping => {
                                println!("Ping received");
                                let _ = tx.send(Ok(json!({ "success": true })));
                            },
                            _ => {
                                println!("Unknown procedure action received");
                            }
                        }
                    }
                }
            }

            println!("Gateway thread ended");
        });

        Gateway {
            join_handle,
            sender: tx,
            turtle_manager,
            block_manager
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