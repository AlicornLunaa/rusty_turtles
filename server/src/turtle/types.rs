use std::fmt;

use futures_util::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// Typings
pub type TurtleSink = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type TurtleSource = SplitStream<WebSocketStream<TcpStream>>;
pub type TurtleSocket = WebSocketStream<TcpStream>;

/// Enums representing various parameters for turtle operations.
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FuelLevel {
    Amount(u32),
    Unlimited,
}

/// Enum for determining tasks on a turtle
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "args")]
pub enum TurtleAction {
    // Movement
    Forward,
    Back,
    Up,
    Down,
    TurnLeft,
    TurnRight,

    // World interactions
    Dig{side: Option<Side>},
    DigUp{side: Option<Side>},
    DigDown{side: Option<Side>},
    Place{text: Option<String>},
    PlaceUp{text: Option<String>},
    PlaceDown{text: Option<String>},
    Attack{side: Option<Side>},
    AttackUp{side: Option<Side>},
    AttackDown{side: Option<Side>},

    // Inventory
    Select{slot: Slot},
    Drop{count: Option<u8>},
    DropUp{count: Option<u8>},
    DropDown{count: Option<u8>},
    Suck{count: Option<u8>},
    SuckUp{count: Option<u8>},
    SuckDown{count: Option<u8>},
    TransferTo{slot: Slot, count: Option<u8>},

    // Fuel & tools
    Refuel{count: Option<u8>},
    EquipLeft,
    EquipRight,

    // Misc
    Craft{limit: Option<u8>},
    Quit,
    StartGpsHost,
    StopGpsHost,
    UpdateLocation{x: i64, y: i64, z: i64, direction: Direction}
}

#[derive(Serialize, Deserialize)]
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