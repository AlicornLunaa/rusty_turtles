use std::fmt;

use futures_util::stream::{SplitSink, SplitStream};
use serde_json::{Value, json};
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// Typings
pub type TurtleSink = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type TurtleSource = SplitStream<WebSocketStream<TcpStream>>;
pub type TurtleSocket = WebSocketStream<TcpStream>;

/// Enums representing various parameters for turtle operations.
pub enum TurtleMessage {
    SendRecv(Value, oneshot::Sender<Result<Value, TurtleError>>),
    Send(Value),
}

#[derive(Clone, Debug)]
pub enum Direction {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

impl Direction {
    pub fn to_value(&self) -> &str {
        match self {
            Direction::NORTH => "north",
            Direction::EAST => "east",
            Direction::SOUTH => "south",
            Direction::WEST => "west",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Side {
    LEFT,
    RIGHT,
}

impl Side {
    pub fn to_value(&self) -> &str {
        match self {
            Side::LEFT => "left",
            Side::RIGHT => "right",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    SLOT1 = 1,
    SLOT2 = 2,
    SLOT3 = 3,
    SLOT4 = 4,
    SLOT5 = 5,
    SLOT6 = 6,
    SLOT7 = 7,
    SLOT8 = 8,
    SLOT9 = 9,
    SLOT10 = 10,
    SLOT11 = 11,
    SLOT12 = 12,
    SLOT13 = 13,
    SLOT14 = 14,
    SLOT15 = 15,
    SLOT16 = 16,
}

impl Slot {
    pub fn to_value(&self) -> Value {
        json!(*self as u8)
    }

    pub fn from_u8(n: u8) -> Self {
        match n {
            1 => Slot::SLOT1,
            2 => Slot::SLOT2,
            3 => Slot::SLOT3,
            4 => Slot::SLOT4,
            5 => Slot::SLOT5,
            6 => Slot::SLOT6,
            7 => Slot::SLOT7,
            8 => Slot::SLOT8,
            9 => Slot::SLOT9,
            10 => Slot::SLOT10,
            11 => Slot::SLOT11,
            12 => Slot::SLOT12,
            13 => Slot::SLOT13,
            14 => Slot::SLOT14,
            15 => Slot::SLOT15,
            16 => Slot::SLOT16,
            _ => Slot::SLOT1,
        }
    }
}

#[derive(Clone, Debug)]
pub enum FuelLevel {
    Amount(u32),
    Unlimited,
}

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
