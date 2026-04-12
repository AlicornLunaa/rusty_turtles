use serde_json::Value;
use tokio::sync::oneshot;

use crate::{gateway::{ServerAction, ServerMessage}, turtle::{self, SmartTurtle, TurtleAction, client::Turtle, types::{Direction, TurtleError}}, util::vector::Vector3};

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

    async fn path_to(&mut self, dest_x: i64, dest_y: i64, dest_z: i64, skip_last: bool) -> Result<(), TurtleError> {
        // Pathfinds to a location, skip_last will stop the turtle from moving to the last spot in the path and instead face it
        let server_tx = self.get_server_tx().clone();

        // Iteratively loop to pathing towards the goal
        loop {
            // Ask server for path
            let (src_x, src_y, src_z) = self.get_position();
            let (tx, rx) = oneshot::channel::<Result<Value, String>>();
            let action = ServerAction::PathTo(src_x, src_y, src_z, dest_x, dest_y, dest_z);
            server_tx.send(ServerMessage::Procedure { client_id: self.get_id(), action, tx }).await.unwrap();
    
            let data = match rx.await {
                Ok(Ok(data)) => data,
                Ok(Err(e)) => return Err(TurtleError::VirtualError(e.to_string())),
                Err(e) => return Err(TurtleError::SocketError(e.to_string())),
            };
    
            let mut path: Vec<Vector3> = serde_json::from_value(data["path"].clone()).unwrap();
    
            // Bail out if no path at all could be found, which means it is unaccessible
            if !data["success"].as_bool().unwrap_or(false) {
                return Err(TurtleError::VirtualError("No viable path.".to_string()));
            }

            // Path to next goal
            let mut sequence = Vec::new();
            let mut previous = Vector3::from(self.get_position());
            let mut sim_direction = self.get_direction();
            let last_spot = path.pop().unwrap_or(Vector3::new(dest_x, dest_y, dest_z));

            for next in &path {
                let delta = *next - previous;
                previous = *next;

                let (mut move_seq, dir) = Turtle::create_movement_commands(delta.x, delta.y, delta.z, sim_direction);
                sequence.append(&mut move_seq);
                sim_direction = dir;
            }

            if skip_last {
                // Just face towards it
                let delta = last_spot - previous;
                let (mut move_seq, _) = Turtle::create_face_commands(sim_direction, delta.x, delta.z);
                sequence.append(&mut move_seq);
            } else {
                // Otherwise, move to it
                let delta = last_spot - previous;
                let (mut move_seq, _) = Turtle::create_movement_commands(delta.x, delta.y, delta.z, sim_direction);
                sequence.append(&mut move_seq);
            }

            // Execute this movement sequence
            let result = self.execute_batch(sequence).await?;

            if !result.success {
                eprintln!("Non-viable path {:?}", result.reason.unwrap_or("error pathing".to_string()));
                self.scan_blocks().await?;
                continue;
            }

            break;
        }

        Ok(())
    }
}
