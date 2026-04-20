use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::{gateway::{ServerAction, ServerMessage}, managers::path_manager::{Coord, ReservedPath}, turtle::{self, Side, Slot, SmartTurtle, client::Turtle, types::{Direction, TurtleError}}, util::vector::Vector3};

/// Enum for determining tasks on a turtle
#[derive(Serialize, Deserialize, Clone)]
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
    Wait,
    StartGpsHost,
    StopGpsHost,
    UpdateLocation{x: i64, y: i64, z: i64, direction: Direction}
}

/// Smart turtle implementation
impl Turtle {
    fn create_face_commands(current_direction: Direction, x: i64, z: i64) -> (Vec<TurtleAction>, Direction) {
        // Get the current direction
        let mut sequence = Vec::new();
        
        // Convert x and z offsets to a target direction
        let new_direction = match (x.signum(), z.signum()) {
            (1, 0) => Direction::EAST,
            (-1, 0) => Direction::WEST,
            (0, -1) => Direction::NORTH,
            (0, 1) => Direction::SOUTH,
            (0, 0) => return (sequence, current_direction), // No offset, no need to turn
            _ => return (sequence, current_direction)
        };

        // Calculate the number of 90-degree right turns needed
        let right_turns = ((new_direction.clone() as i8) - (current_direction as i8)).rem_euclid(4);

        // Execute the most efficient turns
        match right_turns {
            1 => {
                sequence.push(TurtleAction::TurnRight);
            }
            2 => {
                sequence.push(TurtleAction::TurnRight);
                sequence.push(TurtleAction::TurnRight);
            }
            3 => {
                sequence.push(TurtleAction::TurnLeft);
            }
            _ => {}
        }

        (sequence, new_direction)
    }

    fn create_movement_commands(dx: i64, dy: i64, dz: i64, direction: Direction) -> (Vec<TurtleAction>, Direction) {
        let mut sequence = Vec::new();

        for _ in 0..dy.abs() {
            if dy > 0 {
                sequence.push(TurtleAction::Up);
            } else {
                sequence.push(TurtleAction::Down);
            }
        }

        let (mut turn_sequence, direction) = Turtle::create_face_commands(direction, dx, 0);
        sequence.append(&mut turn_sequence);

        for _ in 0..dx.abs() {
            sequence.push(TurtleAction::Forward);
        }

        let (mut turn_sequence, direction) = Turtle::create_face_commands(direction, 0, dz);
        sequence.append(&mut turn_sequence);

        for _ in 0..dz.abs() {
            sequence.push(TurtleAction::Forward);
        }

        (sequence, direction)
    }
}

impl SmartTurtle for Turtle {
    // GPS functions
    async fn start_gps_host(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::StartGpsHost).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("Error hosting GPS".to_string())))
        }
    }

    async fn stop_gps_host(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::StopGpsHost).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("Unspecified error".to_string())))
        }
    }

    // Scanners
    async fn scan_blocks(&self) -> Result<(String, String, String), TurtleError> {
        // This function scans the front, top, and bottom blocks, then tells the server to save them
        let server_tx = self.get_server_tx();
        let (x, y, z) = self.get_position();
        let (fx, fy, fz) = self.get_block_ahead();

        let forward = match self.query(turtle::queries::Inspect).await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        let down = match self.query(turtle::queries::InspectDown).await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        let up = match self.query(turtle::queries::InspectUp).await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(fx, fy, fz, forward.clone())}).await.unwrap();
        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(x, y + 1, z, up.clone())}).await.unwrap();
        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(x, y - 1, z, down.clone())}).await.unwrap();

        Ok((forward.to_string(), up.to_string(), down.to_string()))
    }
    
    // Movement functions
    async fn move_to(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError> {
        // Moves to a spot without pathfinding, first elevation, then x, then z
        let (sequence, _) = Turtle::create_movement_commands(dx, dy, dz, self.get_direction());
        let result = self.execute_batch(sequence).await?;
        
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("Cannot move to coordinates.".to_string())))
        }
    }
}
