#![allow(unused)]
use std::{collections::HashMap, fmt, sync::Arc};

use serde_json::{Value, json};
use tokio::{net::TcpStream, sync::{Mutex, mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};

/// Typings
type TurtleSink = SplitSink<WebSocketStream<TcpStream>, Message>;
type TurtleSource = SplitStream<WebSocketStream<TcpStream>>;
type TurtleSocket = WebSocketStream<TcpStream>;

/// Enums representing various parameters for turtle operations.
#[derive(Clone, Debug)]
pub enum Direction { NORTH, EAST, SOUTH, WEST }
impl Direction {
    fn to_value(&self) -> &str {
        match self {
            Direction::NORTH => "north",
            Direction::EAST => "east",
            Direction::SOUTH => "south",
            Direction::WEST => "west",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Side { LEFT, RIGHT }
impl Side {
    fn to_value(&self) -> &str {
        match self {
            Side::LEFT => "left",
            Side::RIGHT => "right",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot { SLOT1 = 1, SLOT2 = 2, SLOT3 = 3, SLOT4 = 4, SLOT5 = 5, SLOT6 = 6, SLOT7 = 7, SLOT8 = 8, SLOT9 = 9, SLOT10 = 10, SLOT11 = 11, SLOT12 = 12, SLOT13 = 13, SLOT14 = 14, SLOT15 = 15, SLOT16 = 16 }
impl Slot {
    fn to_value(&self) -> Value {
        json!(*self as u8)
    }

    pub fn from_u8(n: u8) -> Self {
        match n {
            1 => Slot::SLOT1, 2 => Slot::SLOT2, 3 => Slot::SLOT3, 4 => Slot::SLOT4,
            5 => Slot::SLOT5, 6 => Slot::SLOT6, 7 => Slot::SLOT7, 8 => Slot::SLOT8,
            9 => Slot::SLOT9, 10 => Slot::SLOT10, 11 => Slot::SLOT11, 12 => Slot::SLOT12,
            13 => Slot::SLOT13, 14 => Slot::SLOT14, 15 => Slot::SLOT15, 16 => Slot::SLOT16,
            _ => Slot::SLOT1,
        }
    }
}

#[derive(Clone, Debug)]
pub enum FuelLevel { Amount(u32), Unlimited }
impl FuelLevel {
    pub fn from_value(v: &Value) -> Self {
        if let Some(n) = v.as_u64() {
            FuelLevel::Amount(n as u32)
        } else if v.as_str() == Some("unlimited") {
            FuelLevel::Unlimited
        } else {
            FuelLevel::Amount(0)
        }
    }
}

/// Error handling enum
#[derive(Debug)]
pub enum TurtleError {
    VirtualError(String), // This is for when an error occured within the turtle world
    SocketError(String), // This is for when an error occured with the socket and the turtle is logically dead
}
impl fmt::Display for TurtleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TurtleError::VirtualError(msg) => write!(f, "Turtle Error: {}", msg),
            TurtleError::SocketError(msg) => write!(f, "Socket Error: {}", msg),
        }
    }
}
impl std::error::Error for TurtleError {}

/// A trait representing the capabilities of a virtual turtle.
pub trait VirtualTurtle {
    // Movement
    async fn forward(&mut self) -> Result<(), TurtleError>;
    async fn back(&mut self) -> Result<(), TurtleError>;
    async fn up(&mut self) -> Result<(), TurtleError>;
    async fn down(&mut self) -> Result<(), TurtleError>;
    async fn turn_left(&mut self) -> Result<(), TurtleError>;
    async fn turn_right(&mut self) -> Result<(), TurtleError>;

    // World Interaction
    async fn dig(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
    async fn place(&mut self, text: Option<String>) -> Result<(), TurtleError>;
    async fn place_up(&mut self, text: Option<String>) -> Result<(), TurtleError>;
    async fn place_down(&mut self, text: Option<String>) -> Result<(), TurtleError>;
    async fn detect(&self) -> Result<bool, TurtleError>;
    async fn detect_up(&self) -> Result<bool, TurtleError>;
    async fn detect_down(&self) -> Result<bool, TurtleError>;
    async fn inspect(&self) -> Result<Value, TurtleError>;
    async fn inspect_up(&self) -> Result<Value, TurtleError>;
    async fn inspect_down(&self) -> Result<Value, TurtleError>;

    // Inventory Management
    async fn select(&mut self, slot: Slot);
    async fn get_selected_slot(&self) -> Slot;
    async fn get_item_count(&self, slot: Option<Slot>) -> u8;
    async fn get_item_space(&self, slot: Option<Slot>) -> u8;
    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<serde_json::Value>, TurtleError>;
    async fn drop(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), TurtleError>;
    async fn compare(&self) -> bool;
    async fn compare_up(&self) -> bool;
    async fn compare_down(&self) -> bool;
    async fn compare_to(&self, slot: Slot) -> bool;

    // Fuel & Upgrades
    async fn get_fuel_level(&self) -> FuelLevel;
    async fn get_fuel_limit(&self) -> FuelLevel;
    async fn refuel(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn equip_left(&mut self) -> Result<(), TurtleError>;
    async fn equip_right(&mut self) -> Result<(), TurtleError>;
    async fn get_equipped_left(&self) -> Result<Option<serde_json::Value>, TurtleError>;
    async fn get_equipped_right(&self) -> Result<Option<serde_json::Value>, TurtleError>;

    // Miscellaneous
    async fn craft(&mut self, limit: Option<u8>) -> Result<(), TurtleError>;
    async fn attack(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), TurtleError>;
}

pub enum TurtleMessage {
    SendRecv(Message, oneshot::Sender<Result<Message, TurtleError>>),
    Send(Message),
}

/// A struct representing a turtle, which implements the VirtualTurtle trait.
pub struct Turtle {
    write_stream: mpsc::Sender<TurtleMessage>,
    join_handle: JoinHandle<()>,
    x: i64,
    y: i64,
    z: i64,
    direction: Direction,
    valid: Arc<Mutex<bool>>
}

impl Turtle {
    // Encapsulation helpers
    fn encapsulate_payload(request_id: u64, message: Value) -> Message {
        let message_text = message.to_string();
        Message::Text(json!({ "request_id": request_id, "message": message_text }).to_string().into())
    }

    fn decapsulate_payload(text: &str) -> (u64, Value) {
        let message: Value = serde_json::from_str(&text).unwrap();
        let request_id = message["request_id"].as_u64().unwrap();
        let message = message["message"].as_str().unwrap();
        (request_id, serde_json::from_str(message).unwrap())
    }

    // Constructors
    async fn initial_handshake(ws_sender: &mut TurtleSink, ws_receiver: &mut TurtleSource) -> Result<(i64, i64, i64, Direction), String> {
        // This function will do the initial questioning for position
        let x;
        let y;
        let z;
        let direction: Direction;
        
        if let Err(e) = ws_sender.send(Turtle::encapsulate_payload(0, json!({ "action": "turtle_init", "args": [] }))).await {
            return Err(format!("Error with sending turtle init. {e}"));
        }

        // Wait for the turtle's response
        if let Some(response) = ws_receiver.next().await {
            match response {
                Ok(Message::Text(text)) => {
                    // Deserialize the returned JSON
                    let (_, data) = Turtle::decapsulate_payload(&text);
                    x = data["x"].as_i64().unwrap();
                    y = data["y"].as_i64().unwrap();
                    z = data["z"].as_i64().unwrap();
                    
                    direction = match data["direction"].as_str() {
                        Some("north") => Direction::NORTH,
                        Some("east") => Direction::EAST,
                        Some("south") => Direction::SOUTH,
                        Some("west") => Direction::WEST,
                        Some(_) => return Err("Invalid direction supplied from turtle.".to_string()),
                        None => return Err("Invalid direction supplied from turtle.".to_string()),
                    };
                },
                Ok(Message::Close(_)) => {
                    return Err(format!("Connection closed prematurely."));
                },
                Err(e) => {
                    eprintln!("Failed to initialize turtle. {e}");
                    return Err(format!("Failed to initialize turtle. {e}"));
                },
                _ => {
                    return Err(format!("Issue with message."));
                },
            }
        } else {
            // No response, likely dropped turtle
            return Err("No response from turtle.".to_string());
        }

        Ok((x, y, z, direction))
    }

    fn spawn_background_thread(mut rx: mpsc::Receiver<TurtleMessage>, mut ws_sender: TurtleSink, mut ws_receiver: TurtleSource, background_valid_flag: Arc<Mutex<bool>>) -> JoinHandle<()> {
        // This function spawns a background thread and takes full ownership of the websocket stream
        tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            let mut pending_responses = HashMap::new();
            let mut request_id: u64 = 0;

            loop {
                tokio::select! {
                    Some(message) = rx.recv() => {
                        // This means the turtle object has sent a message to this, which should be relayed to the websocket
                        match message {
                            TurtleMessage::SendRecv(Message::Text(message), sender) => {
                                // This WILL wait for a message, but first send it to the client
                                let encapsulated = json!({ "request_id": request_id, "message": message.to_string() });

                                if let Err(e) = ws_sender.send(Message::Text(encapsulated.to_string().into())).await {
                                    let _ = sender.send(Err(TurtleError::SocketError(e.to_string())));
                                    break;
                                }

                                // Then wait for the response
                                pending_responses.insert(request_id, sender);
                                request_id += 1;
                            },
                            TurtleMessage::Send(message) => {
                                // Non-blocking for message, it wont wait for a response
                                let encapsulated = json!({ "request_id": request_id, "message": message.to_string() });

                                if ws_sender.send(Message::Text(encapsulated.to_string().into())).await.is_err() {
                                    break;
                                }

                                request_id += 1;
                            },
                            _ => {
                                // Message type not supported
                                break;
                            }
                        }
                    },
                    Some(message) = ws_receiver.next() => {
                        // This means the websocket has received a message, which should be relayed to the server
                        match message {
                            Ok(Message::Text(message)) => {
                                // First decapsulate the data
                                let message: Value = serde_json::from_str(&message).unwrap();
                                let request_id = message["request_id"].as_u64().unwrap();
                                let message = message["message"].as_str().unwrap();
                                let sender = pending_responses.remove(&request_id);

                                match sender {
                                    Some(sender) => {
                                        // Sender found, its a response to a previous message
                                        if sender.send(Ok(Message::Text(message.to_string().into()))).is_err() {
                                            break;
                                        }
                                    },
                                    None => {
                                        // No sender found, this is an unsolicited message
                                        println!("Unsolicited message: {message}");
                                        todo!();
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

    pub async fn new(ws_stream: WebSocketStream<TcpStream>) -> Result<Self, String> {
        // Obtain turtle information via questioning
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<TurtleMessage>(32);
        let valid_flag = Arc::new(Mutex::new(true));

        let (x, y, z, direction) = Self::initial_handshake(&mut ws_sender, &mut ws_receiver).await?;
        let handle = Self::spawn_background_thread(rx, ws_sender, ws_receiver, Arc::clone(&valid_flag));
        
        // Return the turtle object which has the sender object too
        Ok(Self {
            write_stream: tx,
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
    async fn remote_procedure_call(&self, payload: Value) -> Result<Value, TurtleError> {
        // Create JSON table with action and args
        let (tx, rx) = oneshot::channel::<Result<Message, TurtleError>>();

        if let Err(e) = self.write_stream.send(TurtleMessage::SendRecv(Message::Text(payload.to_string().into()), tx)).await {
            // Something went wrong with the write stream
            return Err(TurtleError::SocketError(e.to_string()));
        }

        match rx.await {
            Ok(result) => {
                let response: Value = serde_json::from_str(&result?.to_string()).unwrap();
                Ok(response)
            },
            Err(e) => {
                // The send command failed to obtain a result, probably closed
                *self.valid.lock().await = false;
                Err(TurtleError::SocketError(e.to_string()))
            },
        }
    }

    async fn rpc_success_error(&self, payload: Value) -> Result<Value, TurtleError> {
        let result = self.remote_procedure_call(payload).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(result)
        } else {
            Err(TurtleError::VirtualError(reason.unwrap_or("Unspecified error").to_string()))
        }
    }

    async fn move_relative(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError> {
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
            Err(TurtleError::VirtualError("Failure to save state to turtle?".to_string()))
        }
    }

    async fn rotate_direction(&mut self, direction: Direction) -> Result<(), TurtleError> {
        self.direction = direction;
        self.move_relative(0, 0, 0).await
    }
}

impl Drop for Turtle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

/// Virtual turtle implementation for turtle
impl VirtualTurtle for Turtle {
    async fn forward(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "forward", "args": [] })).await?;

        match self.direction {
            Direction::NORTH => self.move_relative(0, 0, -1).await?,
            Direction::EAST => self.move_relative(1, 0, 0).await?,
            Direction::SOUTH => self.move_relative(0, 0, 1).await?,
            Direction::WEST => self.move_relative(-1, 0, 0).await?,
        };

        Ok(())
    }

    async fn back(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "back", "args": [] })).await?;

        match self.direction {
            Direction::NORTH => self.move_relative(0, 0, 1).await?,
            Direction::EAST => self.move_relative(-1, 0, 0).await?,
            Direction::SOUTH => self.move_relative(0, 0, -1).await?,
            Direction::WEST => self.move_relative(1, 0, 0).await?,
        };
        
        Ok(())
    }

    async fn up(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "up", "args": [] })).await?;
        self.move_relative(0, 1, 0).await?;
        Ok(())
    }

    async fn down(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "down", "args": [] })).await?;
        self.move_relative(0, -1, 0).await?;
        Ok(())
    }

    async fn turn_left(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "turnLeft", "args": [] })).await?;

        match self.direction {
            Direction::NORTH => self.rotate_direction(Direction::WEST).await?,
            Direction::EAST => self.rotate_direction(Direction::NORTH).await?,
            Direction::SOUTH => self.rotate_direction(Direction::EAST).await?,
            Direction::WEST => self.rotate_direction(Direction::SOUTH).await?,
        }
        
        Ok(())
    }

    async fn turn_right(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "turnRight", "args": [] })).await?;

        match self.direction {
            Direction::NORTH => self.rotate_direction(Direction::EAST).await?,
            Direction::EAST => self.rotate_direction(Direction::SOUTH).await?,
            Direction::SOUTH => self.rotate_direction(Direction::WEST).await?,
            Direction::WEST => self.rotate_direction(Direction::NORTH).await?,
        };

        Ok(())
    }

    async fn dig(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "dig", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }

    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "digUp", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }

    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "digDown", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }

    async fn place(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "place", "args": [text] })).await?;
        Ok(())
    }

    async fn place_up(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "placeUp", "args": [text] })).await?;
        Ok(())
    }

    async fn place_down(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "placeDown", "args": [text] })).await?;
        Ok(())
    }

    async fn detect(&self) -> Result<bool, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "detect", "args": [] })).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    async fn detect_up(&self) -> Result<bool, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "detectUp", "args": [] })).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    async fn detect_down(&self) -> Result<bool, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "detectDown", "args": [] })).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    async fn inspect(&self) -> Result<Value, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "inspect", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        
        if success {
            Ok(result["data"].clone())
        } else {
            let reason = result["data"].as_str().unwrap_or("Unknown error");
            Err(TurtleError::VirtualError(reason.to_string()))
        }
    }

    async fn inspect_up(&self) -> Result<Value, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "inspectUp", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);

        if success {
            Ok(result["data"].clone())
        } else {
            let reason = result["data"].as_str().unwrap_or("Unknown error");
            Err(TurtleError::VirtualError(reason.to_string()))
        }
    }

    async fn inspect_down(&self) -> Result<Value, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "inspectDown", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);

        if success {
            Ok(result["data"].clone())
        } else {
            let reason = result["data"].as_str().unwrap_or("Unknown error");
            Err(TurtleError::VirtualError(reason.to_string()))
        }
    }

    async fn select(&mut self, slot: Slot) {
        let _ = self.remote_procedure_call(json!({ "action": "select", "args": [slot as u8] })).await;
    }

    async fn get_selected_slot(&self) -> Slot {
        let result = self.remote_procedure_call(json!({ "action": "getSelectedSlot", "args": [] })).await.unwrap_or(json!({ "slot": 1 }));
        Slot::from_u8(result["slot"].as_u64().unwrap() as u8)
    }

    async fn get_item_count(&self, slot: Option<Slot>) -> u8 {
        let result = self.remote_procedure_call(json!({ "action": "getItemCount", "args": [slot.map(|s| s as u8)] })).await.unwrap_or(json!({ "count": 0 }));
        result["count"].as_u64().unwrap() as u8
    }

    async fn get_item_space(&self, slot: Option<Slot>) -> u8 {
        let result = self.remote_procedure_call(json!({ "action": "getItemSpace", "args": [slot.map(|s| s as u8)] })).await.unwrap_or(json!({ "space": 64 }));
        result["space"].as_u64().unwrap() as u8
    }

    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<Value>, TurtleError> {
        let args = json!([slot.map(|s| s as u8), detailed]);
        let result = self.remote_procedure_call(json!({ "action": "getItemDetail", "args": args })).await?;
        Ok(if result["detail"].is_null() { None } else { Some(result["detail"].clone()) })
    }

    async fn drop(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "drop", "args": [count] })).await?;
        Ok(())
    }

    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "dropUp", "args": [count] })).await?;
        Ok(())
    }

    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "dropDown", "args": [count] })).await?;
        Ok(())
    }

    async fn suck(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "suck", "args": [count] })).await?;
        Ok(())
    }

    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "suckUp", "args": [count] })).await?;
        Ok(())
    }

    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "suckDown", "args": [count] })).await?;
        Ok(())
    }

    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "transferTo", "args": [slot as u8, count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError("Number of items out of range.".to_string()))
        }
    }

    async fn compare(&self) -> bool {
        let result = self.remote_procedure_call(json!({ "action": "compare", "args": [] })).await.unwrap_or(json!({ "data": false }));
        result["data"].as_bool().unwrap()
    }

    async fn compare_up(&self) -> bool {
        let result = self.remote_procedure_call(json!({ "action": "compareUp", "args": [] })).await.unwrap_or(json!({ "data": false }));
        result["data"].as_bool().unwrap()
    }

    async fn compare_down(&self) -> bool {
        let result = self.remote_procedure_call(json!({ "action": "compareDown", "args": [] })).await.unwrap_or(json!({ "data": false }));
        result["data"].as_bool().unwrap()
    }

    async fn compare_to(&self, slot: Slot) -> bool {
        let result = self.remote_procedure_call(json!({ "action": "compareTo", "args": [slot as u8] })).await.unwrap_or(json!({ "data": false }));
        result["data"].as_bool().unwrap()
    }

    async fn get_fuel_level(&self) -> FuelLevel {
        let result = self.remote_procedure_call(json!({ "action": "getFuelLevel", "args": [] })).await.unwrap_or(json!({ "level": 0 }));
        FuelLevel::from_value(&result["level"])
    }

    async fn get_fuel_limit(&self) -> FuelLevel {
        let result = self.remote_procedure_call(json!({ "action": "getFuelLimit", "args": [] })).await.unwrap_or(json!({ "limit": 0 }));
        FuelLevel::from_value(&result["limit"])
    }

    async fn refuel(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "refuel", "args": [count] })).await?;
        Ok(())
    }

    async fn equip_left(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "equipLeft", "args": [] })).await?;
        Ok(())
    }

    async fn equip_right(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "equipRight", "args": [] })).await?;
        Ok(())
    }

    async fn get_equipped_left(&self) -> Result<Option<serde_json::Value>, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "getEquippedLeft", "args": [] })).await?;
        Ok(if result["detail"].is_null() { None } else { Some(result["detail"].clone()) })
    }

    async fn get_equipped_right(&self) -> Result<Option<serde_json::Value>, TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "getEquippedRight", "args": [] })).await?;
        Ok(if result["detail"].is_null() { None } else { Some(result["detail"].clone()) })
    }

    async fn craft(&mut self, limit: Option<u8>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "craft", "args": [limit] })).await?;
        Ok(())
    }

    async fn attack(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "attack", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }

    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "attackUp", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }

    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "attackDown", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        Ok(())
    }
}