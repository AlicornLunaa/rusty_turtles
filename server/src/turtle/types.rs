use std::fmt;

use futures_util::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::turtle::TurtleAction;

/// Typings
pub type TurtleSink = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type TurtleSource = SplitStream<WebSocketStream<TcpStream>>;
pub type TurtleSocket = WebSocketStream<TcpStream>;

/// Enums representing various parameters for turtle operations.
#[derive(Debug)]
pub enum TurtleMessage {
    Action { actions: Vec<TurtleAction>, return_tx: Option<oneshot::Sender<TurtleResponse>> },
    Query { query: Value, response: oneshot::Sender<TurtleResponse> }
}

#[repr(u8)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Direction {
    NORTH = 0,
    EAST = 1,
    SOUTH = 2,
    WEST = 3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Side {
    LEFT,
    RIGHT,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
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

pub const SLOTS: [Slot; 16] = [Slot::SLOT1, Slot::SLOT2, Slot::SLOT3, Slot::SLOT4, Slot::SLOT5, Slot::SLOT6, Slot::SLOT7, Slot::SLOT8, Slot::SLOT9, Slot::SLOT10, Slot::SLOT11, Slot::SLOT12, Slot::SLOT13, Slot::SLOT14, Slot::SLOT15, Slot::SLOT16];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FuelLevel {
    Amount(u32),
    Unlimited,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TurtleResponse {
    pub success: bool, // if the command failed
    pub reason: Option<String>, // Why the command failed
    pub last_action: u64, // This is the last action performed in the array
    pub data: Option<Value>, // Used to send back complex data
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TurtlePayload {
    Procedure{ id: u64, data: Vec<TurtleAction> },
    Oneshot{ id: u64, data: Vec<TurtleAction> },
    Query{ id: u64, data: Value },
    Response{ id: u64, data: TurtleResponse }
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