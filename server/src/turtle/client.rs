use std::{collections::HashMap, sync::Arc};

use serde_json::{Value, json};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::{Mutex, mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{gateway::{ServerAction, ServerMessage}, payload::Payload, turtle::types::*};

/// A struct representing a turtle, which implements the VirtualTurtle trait.
pub struct Turtle {
    id: u64,
    turtle_write_stream: mpsc::Sender<TurtleMessage>,
    join_handle: JoinHandle<()>,
    x: i64,
    y: i64,
    z: i64,
    direction: Direction,
    valid: Arc<Mutex<bool>>
}

impl Turtle {
    // Constructors
    fn spawn_worker_thread(client_id: u64, mut turtle_rx: mpsc::Receiver<TurtleMessage>, server_tx: mpsc::Sender<ServerMessage>, mut ws_sender: TurtleSink, mut ws_receiver: TurtleSource, background_valid_flag: Arc<Mutex<bool>>) -> JoinHandle<()> {
        // This function spawns a background thread and takes full ownership of the websocket stream
        tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            let mut pending_responses = HashMap::new();
            let mut request_id: u64 = 0;

            loop {
                tokio::select! {
                    Some(message) = turtle_rx.recv() => {
                        // This means the server has sent a message to this turtle, which should be relayed to the websocket
                        match message {
                            TurtleMessage::SendRecv(data, sender) => {
                                // This tells the server to expect a response after it sends, like a procedure call
                                let payload = Payload::Request { id: request_id, oneshot: false, data };

                                if let Err(e) = ws_sender.send(Message::Text(payload.encode().into())).await {
                                    let _ = sender.send(Err(TurtleError::SocketError(e.to_string())));
                                    break;
                                }

                                // Then wait for the response
                                pending_responses.insert(request_id, sender);
                                request_id += 1;
                            },
                            TurtleMessage::Send(data) => {
                                // Non-blocking for message, it wont wait for a response
                                let payload = Payload::Request { id: request_id, oneshot: true, data };

                                if ws_sender.send(Message::Text(payload.encode().into())).await.is_err() {
                                    break;
                                }

                                request_id += 1;
                            },
                        }
                    },
                    Some(message) = ws_receiver.next() => {
                        // This means the turtle has sent a message to the server/websocket, which should be relayed to the gateway or used as a response
                        match message {
                            Ok(Message::Text(message)) => {
                                // First decapsulate the data
                                let payload = Payload::decode(&message).unwrap();

                                match payload {
                                    Payload::Request { id, oneshot, data } => {
                                        // This is a new request for the server from the turtle. This one is blocking for the turtle worker thread
                                        let action = match data["action"].as_str() {
                                            Some("ping") => ServerAction::Ping,
                                            _ => continue,
                                        };
                                        
                                        // Execute oneshot waiting
                                        if !oneshot {
                                            // This isn't a one shot request, it needs a return channel to wait for
                                            let (tx, rx) = oneshot::channel::<Result<Value, String>>();
                                            let server_message = ServerMessage::Procedure { client_id: client_id, action, tx };
                                            
                                            if let Err(e) = server_tx.send(server_message).await {
                                                // Something went wrong with the write stream
                                                let payload = Payload::Response { id, data: json!({ "success": false, "error": e.to_string() }) };

                                                if ws_sender.send(Message::Text(payload.encode().into())).await.is_err() {
                                                    break;
                                                }
                                            }

                                            // Wait for immediate response
                                            let payload = match rx.await {
                                                Ok(Ok(data)) => Payload::Response { id, data },
                                                Ok(Err(e)) => Payload::Response { id, data: json!({ "success": false, "error": e }) },
                                                Err(e) => Payload::Response { id, data: json!({ "success": false, "error": e.to_string() }) },
                                            };

                                            if ws_sender.send(Message::Text(payload.encode().into())).await.is_err() {
                                                break;
                                            }
                                        } else {
                                            // This is a one shot, so just send it off and forget
                                            let server_message = ServerMessage::Oneshot { client_id: client_id, action };
                                            let _ = server_tx.send(server_message).await;
                                        }
                                    },
                                    Payload::Response { id, data } => {
                                        // This is a response from the turtle to the server
                                        let sender = pending_responses.remove(&id);

                                        if let Some(sender) = sender {
                                            // Send the response to the sender if the channel still exists
                                            if sender.send(Ok(data)).is_err() {
                                                // If the response channel doesn't work, we've probably dropped the turtle.
                                                break;
                                            }
                                        }
                                    },
                                }
                            },
                            _ => {
                                break;
                            }
                        }
                    }
                }
            }

            *background_valid_flag.lock().await = false;
            println!("Consumer lost, closing thread");
        })
    }

    async fn initial_handshake(turtle_tx: mpsc::Sender<TurtleMessage>) -> Result<(i64, i64, i64, Direction), String> {
        // Send request to the turtle
        let (tx, rx) = oneshot::channel::<Result<Value, TurtleError>>();

        if let Err(e) = turtle_tx.send(TurtleMessage::SendRecv(json!({ "action": "turtle_init", "args": [] }).into(), tx)).await {
            // Something went wrong with the write stream
            return Err(e.to_string());
        }

        // Wait for the data back
        match rx.await {
            Ok(result) => {
                if result.is_err() {
                    return Err("No result from turtle's handshake".to_string());
                }

                let result = result.unwrap();
                let x = result["x"].as_i64().unwrap();
                let y = result["y"].as_i64().unwrap();
                let z = result["z"].as_i64().unwrap();
                
                let direction = match result["direction"].as_str() {
                    Some("north") => Direction::NORTH,
                    Some("east") => Direction::EAST,
                    Some("south") => Direction::SOUTH,
                    Some("west") => Direction::WEST,
                    Some(_) => return Err("Invalid direction supplied from turtle.".to_string()),
                    None => return Err("Invalid direction supplied from turtle.".to_string()),
                };

                return Ok((x, y, z, direction));
            },
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                Err(e.to_string())
            },
        }
    }

    pub async fn new(id: u64, ws_stream: WebSocketStream<TcpStream>, server_tx: mpsc::Sender<ServerMessage>) -> Result<Self, String> {
        // Obtain turtle information via questioning
        let (ws_sender, ws_receiver) = ws_stream.split();
        let (tx, rx) = mpsc::channel::<TurtleMessage>(32);
        let valid_flag = Arc::new(Mutex::new(true));

        let handle = Self::spawn_worker_thread(id, rx, server_tx, ws_sender, ws_receiver, Arc::clone(&valid_flag));
        let (x, y, z, direction) = Self::initial_handshake(tx.clone()).await?;
        
        // Return the turtle object which has the sender object too
        Ok(Self {
            id,
            turtle_write_stream: tx,
            join_handle: handle,
            x, y, z,
            direction,
            valid: valid_flag
        })
    }

    // Turtle getters
    pub async fn is_valid(&self) -> bool {
        *self.valid.lock().await
    }

    pub fn get_position(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    pub fn get_direction(&self) -> Direction {
        self.direction.clone()
    }

    // Private helpers for communication with the actual turtle
    pub async fn remote_procedure_call(&self, payload: Value) -> Result<Value, TurtleError> {
        // Create JSON table with action and args
        let (tx, rx) = oneshot::channel::<Result<Value, TurtleError>>();

        if let Err(e) = self.turtle_write_stream.send(TurtleMessage::SendRecv(payload, tx)).await {
            // Something went wrong with the write stream
            return Err(TurtleError::SocketError(e.to_string()));
        }

        match rx.await {
            Ok(result) => {
                Ok(result?)
            },
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                *self.valid.lock().await = false;
                Err(TurtleError::SocketError(e.to_string()))
            },
        }
    }

    pub async fn rpc_success_error(&self, payload: Value) -> Result<Value, TurtleError> {
        let result = self.remote_procedure_call(payload).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(result)
        } else {
            Err(TurtleError::VirtualError(reason.unwrap_or("Unspecified error").to_string()))
        }
    }

    pub(super) async fn move_relative(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError> {
        // Update by deltas
        self.x += dx;
        self.y += dy;
        self.z += dz;

        // Update turtle with another RPC
        let result = self.remote_procedure_call(json!({ "action": "update_location", "args": [self.x, self.y, self.z, self.direction.to_value()] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        // Error handling
        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap_or("Failure to save state to turtle?").to_string()))
        }
    }

    pub(super) async fn rotate_direction(&mut self, direction: Direction) -> Result<(), TurtleError> {
        self.direction = direction;
        self.move_relative(0, 0, 0).await
    }
}

impl Drop for Turtle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}