use std::sync::Arc;

use serde_json::{Value, json};
use tokio::{sync::{Mutex, mpsc, oneshot}, task::JoinHandle};

use crate::{managers::{block_manager::BlockManager, turtle_manager::{self, TurtleManager}}, pathfinding, util::vector::Vector3};
use crate::pathfinding::pathfinding::find_path;

/// This module contains the server controller for incoming requests
pub enum ServerAction {
    Ping,
    SetupGPS,
    StopGPS,
    UpdateBlock(i64, i64, i64, String),
    PathTo(i64, i64, i64, i64, i64, i64),
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
    fn ping_oneshot(id: u64){
        println!("Ping received from {id}");
    }

    async fn start_gps_procedure(){
        // This procedure finds 4 turtles which are not in use, tells them to move out into a constellation, then host GPS
        // for another turtle to locate itself for bootstrapping
    }

    pub fn new(turtle_manager: Arc<Mutex<TurtleManager>>, block_manager: Arc<Mutex<BlockManager>>) -> Self {
        // Start a MPSC channel to handle incoming requests
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn gateway thread to handle incoming requests
        let join_handle = tokio::spawn({
            let turtle_manager = Arc::clone(&turtle_manager);
            let block_manager = Arc::clone(&block_manager);

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
                                        block_manager.lock().await.update_block(x, y, z, block).await.unwrap();
                                    } else {
                                        block_manager.lock().await.remove_block(x, y, z).await.unwrap();
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
                                ServerAction::PathTo(x1, y1, z1, x2, y2, z2) => {
                                    let block_manager = block_manager.lock().await;
                                    let res = find_path(&*block_manager, Vector3::new(x1, y1, z1), Vector3::new(x2, y2, z2)).await;
                                    let _ = tx.send(Ok(json!({ "success": res.is_some(), "path": res.unwrap_or(Vec::new()) })));
                                },
                                _ => {
                                    println!("Unknown procedure action received");
                                }
                            }
                        }
                    }
                }

                println!("Gateway thread ended");
            }
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