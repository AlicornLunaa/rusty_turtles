use std::{fmt, sync::Arc};

use serde_json::{Value, json};
use tokio::{net::TcpStream, sync::{Mutex, mpsc, oneshot}, task::JoinHandle};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

/// Enums representing various parameters for turtle operations.
#[derive(Clone, Debug)]
pub enum Direction { NORTH, EAST, SOUTH, WEST }

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
    pub async fn new(ws_stream: WebSocketStream<TcpStream>) -> Result<Self, String> {
        // Obtain turtle information via questioning
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let x;
        let y;
        let z;
        let direction: Direction;

        if let Err(e) = ws_sender.send(Message::Text(json!({ "action": "turtle_init", "args": [] }).to_string().into())).await {
            return Err(format!("Error with sending turtle init. {e}"));
        }

        // Wait for the turtle's response
        if let Some(response) = ws_receiver.next().await {
            match response {
                Ok(Message::Text(text)) => {
                    // Deserialize the returned JSON
                    let data: Value = serde_json::from_str(&text).unwrap();
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
                    }
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

        // Keep track of the turtle's status across multiple threads
        let valid_flag = Arc::new(Mutex::new(true));
        
        // Spawn generic controller for all messages
        let (tx, mut rx) = mpsc::channel::<TurtleMessage>(32);
        let background_valid_flag = Arc::clone(&valid_flag);

        let handle = tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            while let Some(message) = rx.recv().await {
                match message {
                    TurtleMessage::SendRecv(message, sender) => {
                        // This WILL wait for a message, but first send it to the client
                        if let Err(e) = ws_sender.send(message).await {
                            let _ = sender.send(Err(TurtleError::SocketError(e.to_string())));
                            break;
                        }

                        // Then wait for the response
                        match ws_receiver.next().await {
                            Some(Ok(Message::Text(text))) => {
                                // Actual response message
                                if sender.send(Ok(Message::Text(text))).is_err() {
                                    break;
                                }
                            },
                            Some(Ok(Message::Binary(bin))) => {
                                // Actual response message
                                if sender.send(Ok(Message::Binary(bin))).is_err() {
                                    break;
                                }
                            },
                            _ => {
                                // Error given by ws_receiver, client likely gone
                                let _ = sender.send(Err(TurtleError::SocketError("Something went wrong with the oneshot channel.".to_string())));
                                break;
                            },
                        }
                    },
                    TurtleMessage::Send(message) => {
                        // Non-blocking for message, it wont wait for a response
                        if ws_sender.send(message).await.is_err() {
                            break;
                        }
                    },
                }
            }

            *background_valid_flag.lock().await = false;
            println!("Consumer lost, closing thread");
        });

        // Return the turtle object which has the sender object too
        Ok(Self {
            write_stream: tx,
            join_handle: handle,
            x, y, z,
            direction,
            valid: valid_flag
        })
    }

    pub fn get_position(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    pub fn get_direction(&self) -> Direction {
        self.direction.clone()
    }

    pub async fn is_valid(&self) -> bool {
        *self.valid.lock().await
    }

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
}

impl Drop for Turtle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

/// Virtual turtle implementation for turtle
impl VirtualTurtle for Turtle {
    async fn forward(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "forward", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn back(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "back", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn up(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "up", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn down(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "down", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn turn_left(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "turnLeft", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn turn_right(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "turnRight", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn dig(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "dig", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "digUp", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();
        
        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "digDown", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn place(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "place", "args": [text] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn place_up(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "placeUp", "args": [text] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn place_down(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "placeDown", "args": [text] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
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
        let result = self.remote_procedure_call(json!({ "action": "drop", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "dropUp", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "dropDown", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn suck(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "suck", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "suckUp", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "suckDown", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
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
        let result = self.remote_procedure_call(json!({ "action": "refuel", "args": [count] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn equip_left(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "equipLeft", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn equip_right(&mut self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "equipRight", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
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
        let result = self.remote_procedure_call(json!({ "action": "craft", "args": [limit] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn attack(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "attack", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "attackUp", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }

    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "attackDown", "args": [side.map(|s| s.to_value().to_string())] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap().to_string()))
        }
    }
}