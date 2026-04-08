use std::{collections::VecDeque, sync::mpsc::RecvError};

use serde_json::Value;
use tokio::{net::TcpStream, sync::{mpsc, oneshot}};
use tokio_tungstenite::{WebSocketStream, tungstenite::{Error, Message}};
use futures_util::{SinkExt, StreamExt, TryStreamExt, stream::SplitSink};

/// Enums representing various parameters for turtle operations.
#[derive(Clone, Debug)]
pub enum Direction { NORTH, EAST, SOUTH, WEST }
#[derive(Clone, Debug)]
pub enum Side { LEFT, RIGHT }
#[derive(Clone, Debug)]
pub enum Slot { SLOT1 = 1, SLOT2 = 2, SLOT3 = 3, SLOT4 = 4, SLOT5 = 5, SLOT6 = 6, SLOT7 = 7, SLOT8 = 8, SLOT9 = 9, SLOT10 = 10, SLOT11 = 11, SLOT12 = 12, SLOT13 = 13, SLOT14 = 14, SLOT15 = 15, SLOT16 = 16 }
#[derive(Clone, Debug)]
pub enum FuelLevel { Amount(u32), Unlimited }

/// A trait representing the capabilities of a virtual turtle.
pub trait VirtualTurtle {
    // Movement
    async fn forward(&mut self) -> Result<(), String>;
    async fn back(&mut self) -> Result<(), String>;
    async fn up(&mut self) -> Result<(), String>;
    async fn down(&mut self) -> Result<(), String>;
    async fn turn_left(&mut self) -> Result<(), String>;
    async fn turn_right(&mut self) -> Result<(), String>;

    // World Interaction
    async fn dig(&mut self, side: Option<Side>) -> Result<(), String>;
    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), String>;
    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), String>;
    async fn place(&mut self, text: Option<String>) -> Result<(), String>;
    async fn place_up(&mut self, text: Option<String>) -> Result<(), String>;
    async fn place_down(&mut self, text: Option<String>) -> Result<(), String>;
    async fn detect(&self) -> bool;
    async fn detect_up(&self) -> bool;
    async fn detect_down(&self) -> bool;
    async fn inspect(&self) -> (bool, String);
    async fn inspect_up(&self) -> (bool, String);
    async fn inspect_down(&self) -> (bool, String);

    // Inventory Management
    async fn select(&mut self, slot: Slot);
    async fn get_selected_slot(&self) -> Slot;
    async fn get_item_count(&self, slot: Option<Slot>) -> u8;
    async fn get_item_space(&self, slot: Option<Slot>) -> u8;
    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<serde_json::Value>, String>;
    async fn drop(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn suck(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), String>;
    async fn compare(&self) -> bool;
    async fn compare_up(&self) -> bool;
    async fn compare_down(&self) -> bool;
    async fn compare_to(&self, slot: Slot) -> bool;

    // Fuel & Upgrades
    async fn get_fuel_level(&self) -> FuelLevel;
    async fn get_fuel_limit(&self) -> FuelLevel;
    async fn refuel(&mut self, count: Option<u8>) -> Result<(), String>;
    async fn equip_left(&mut self) -> Result<(), String>;
    async fn equip_right(&mut self) -> Result<(), String>;
    async fn get_equipped_left(&self) -> Result<Option<serde_json::Value>, String>;
    async fn get_equipped_right(&self) -> Result<Option<serde_json::Value>, String>;

    // Miscellaneous
    async fn craft(&mut self, limit: Option<u8>) -> Result<(), String>;
    async fn attack(&mut self, side: Option<Side>) -> Result<(), String>;
    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), String>;
    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), String>;
}

pub enum TurtleMessage {
    SendRecv(Message, oneshot::Sender<Result<Message, Error>>),
    Send(Message),
}

/// A struct representing a turtle, which implements the VirtualTurtle trait.
pub struct Turtle {
    write_stream: mpsc::Sender<TurtleMessage>,
    valid: bool,
    x: i64,
    y: i64,
    z: i64,
    direction: Direction,
}

impl Turtle {
    pub async fn new(ws_stream: WebSocketStream<TcpStream>) -> Result<Self, String> {
        // Obtain turtle information via questioning
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let mut x = 0;
        let mut y = 0;
        let mut z = 0;
        let mut direction = Direction::NORTH;

        ws_sender.send(Message::Text("turtle_init".into()));

        if let Some(response) = ws_receiver.next().await {
            match response {
                Ok(message) => {
                    // Deserialize the returned JSON
                    let data: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
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
                Err(e) => {
                    eprintln!("Failed to initialize turtle. {e}");
                    return Err(format!("Failed to initialize turtle. {e}"));
                },
            }
        }
        
        // Spawn generic controller for all messages
        let (tx, mut rx) = mpsc::channel::<TurtleMessage>(32);

        tokio::spawn(async move {
            // This background thread manages messages from tx by reading rx and sending it to the socket
            println!("Started consumer thread");

            while let Some(message) = rx.recv().await {
                match message {
                    TurtleMessage::SendRecv(message, sender) => {
                        // This WILL wait for a message
                        ws_sender.send(message);

                        if let Some(response) = ws_receiver.next().await {
                            sender.send(response);
                        }
                    },
                    TurtleMessage::Send(message) => {
                        // Non-blocking for message, it wont wait for a response
                        ws_sender.send(message);
                    },
                }
            }

            println!("Consumer lost, closing thread");
        });

        // Return the turtle object which has the sender object too
        Ok(Self {
            write_stream: tx,
            valid: true,
            x, y, z,
            direction,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_position(&self) -> (i64, i64, i64) {
        (self.x, self.y, self.z)
    }

    pub fn get_direction(&self) -> Direction {
        self.direction.clone()
    }
}

/// Virtual turtle implementation for turtle
impl VirtualTurtle for Turtle {
    async fn forward(&mut self) -> Result<(), String> {
        // Create JSON table with action and args
        let (tx, rx) = oneshot::channel::<Result<Message, Error>>();
        let payload = serde_json::json!({ "action": "forward", "args": [] });
        self.write_stream.send(TurtleMessage::SendRecv(Message::Text(payload.to_string().into()), tx));

        match rx.await {
            Ok(result) => {
                let response: Value = serde_json::from_str(&result.unwrap().to_string()).unwrap();
                let success = response["success"].as_bool().unwrap();
                let reason = response["reason"].as_str();

                if success {
                    Ok(())
                } else {
                    Err(reason.unwrap().to_string())
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }

    async fn back(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn up(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn down(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn turn_left(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn turn_right(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn dig(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }

    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }

    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }

    async fn place(&mut self, text: Option<String>) -> Result<(), String> {
        todo!()
    }

    async fn place_up(&mut self, text: Option<String>) -> Result<(), String> {
        todo!()
    }

    async fn place_down(&mut self, text: Option<String>) -> Result<(), String> {
        todo!()
    }

    async fn detect(&self) -> bool {
        todo!()
    }

    async fn detect_up(&self) -> bool {
        todo!()
    }

    async fn detect_down(&self) -> bool {
        todo!()
    }

    async fn inspect(&self) -> (bool, String) {
        todo!()
    }

    async fn inspect_up(&self) -> (bool, String) {
        todo!()
    }

    async fn inspect_down(&self) -> (bool, String) {
        todo!()
    }

    async fn select(&mut self, slot: Slot) {
        todo!()
    }

    async fn get_selected_slot(&self) -> Slot {
        todo!()
    }

    async fn get_item_count(&self, slot: Option<Slot>) -> u8 {
        todo!()
    }

    async fn get_item_space(&self, slot: Option<Slot>) -> u8 {
        todo!()
    }

    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<serde_json::Value>, String> {
        todo!()
    }

    async fn drop(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn suck(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn compare(&self) -> bool {
        todo!()
    }

    async fn compare_up(&self) -> bool {
        todo!()
    }

    async fn compare_down(&self) -> bool {
        todo!()
    }

    async fn compare_to(&self, slot: Slot) -> bool {
        todo!()
    }

    async fn get_fuel_level(&self) -> FuelLevel {
        todo!()
    }

    async fn get_fuel_limit(&self) -> FuelLevel {
        todo!()
    }

    async fn refuel(&mut self, count: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn equip_left(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn equip_right(&mut self) -> Result<(), String> {
        todo!()
    }

    async fn get_equipped_left(&self) -> Result<Option<serde_json::Value>, String> {
        todo!()
    }

    async fn get_equipped_right(&self) -> Result<Option<serde_json::Value>, String> {
        todo!()
    }

    async fn craft(&mut self, limit: Option<u8>) -> Result<(), String> {
        todo!()
    }

    async fn attack(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }

    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }

    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), String> {
        todo!()
    }
}