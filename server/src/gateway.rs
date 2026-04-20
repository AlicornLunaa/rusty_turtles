use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};

use crate::{managers::{block_manager::BlockManager, path_manager::{PathManager, ReservedPath}, turtle_manager::TurtleManager}, util::vector::Vector3};

/// This module contains the server controller for incoming requests
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum ServerAction {
    Ping,
    SetupGPS,
    StopGPS,
    UpdateBlock(i64, i64, i64, String),
}

pub enum ServerMessage {
    Procedure{ client_id: u64, action: ServerAction, tx: oneshot::Sender<Result<Value, String>>}, // The caller expects a response
    Oneshot{ client_id: u64, action: ServerAction }, // An action fired off and forgot about
    ReservePath{ client_id: u64, x1: i64, y1: i64, z1: i64, x2: i64, y2: i64, z2: i64, reply: oneshot::Sender<Option<ReservedPath>> }
}

pub struct Gateway {
    join_handle: JoinHandle<()>,
    sender: mpsc::Sender<ServerMessage>, // Used for cloning
    turtle_manager: TurtleManager,
    block_manager: BlockManager,
    path_ledger: PathManager,
}

impl Gateway {
    fn ping_oneshot(id: u64){
        println!("Ping received from {id}");
    }

    pub fn new(turtle_manager: TurtleManager, block_manager: BlockManager, path_ledger: PathManager) -> Self {
        // Start a MPSC channel to handle incoming requests
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn gateway thread to handle incoming requests
        let join_handle = tokio::spawn({
            let block_manager = block_manager.clone();
            let path_ledger = path_ledger.clone();

            async move {
                println!("Starting gateway thread");

                while let Some(message) = rx.recv().await {
                    match message {
                        ServerMessage::Oneshot { client_id, action } => {
                            match action {
                                ServerAction::Ping => {
                                    Gateway::ping_oneshot(client_id);
                                },
                                ServerAction::UpdateBlock(x, y, z, block) => {
                                    if block != "minecraft:air" {
                                        block_manager.update_block(x, y, z, block).await;
                                    } else {
                                        block_manager.remove_block(x, y, z).await;
                                    }
                                },
                                _ => {
                                    println!("Unknown oneshot action received");
                                }
                            }
                        },
                        ServerMessage::Procedure { client_id, action, tx } => {
                            match action {
                                ServerAction::Ping => {
                                    Gateway::ping_oneshot(client_id);
                                    let _ = tx.send(Ok(json!({ "success": true })));
                                },
                                _ => {
                                    println!("Unknown procedure action received");
                                }
                            }
                        }
                        ServerMessage::ReservePath { client_id, x1, y1, z1, x2, y2, z2, reply } => {
                            // Use WHCA* via path_ledger
                            let result = path_ledger.path_to(
                                client_id, 
                                Vector3::new(x1, y1, z1), 
                                Vector3::new(x2, y2, z2), 
                                32 // Default window size
                            ).await;

                            if let Ok(reservation) = result {
                                let _ = reply.send(Some(reservation));
                            } else {
                                let _ = reply.send(None);
                            }
                        },
                    }
                }

                println!("Gateway thread ended");
            }
        });

        Gateway {
            join_handle,
            sender: tx,
            turtle_manager,
            block_manager,
            path_ledger,
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