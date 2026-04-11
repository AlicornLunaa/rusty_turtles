use serde_json::Value;
use tokio::sync::oneshot;

use crate::{gateway::{ServerAction, ServerMessage}, turtle::{self, SmartTurtle, TurtleAction, client::Turtle, traits::VirtualTurtle, types::{Direction, FuelLevel, Side, Slot, TurtleError}}, util::vector::Vector3};

/// Virtual turtle implementation for turtle
impl VirtualTurtle for Turtle {
    async fn forward(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Forward).await?;

        if result.success {
            match self.get_direction() {
                Direction::NORTH => self.move_relative(0, 0, -1).await?,
                Direction::EAST => self.move_relative(1, 0, 0).await?,
                Direction::SOUTH => self.move_relative(0, 0, 1).await?,
                Direction::WEST => self.move_relative(-1, 0, 0).await?,
            };
    
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn back(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Back).await?;

        if result.success {
            match self.get_direction() {
                Direction::NORTH => self.move_relative(0, 0, 1).await?,
                Direction::EAST => self.move_relative(-1, 0, 0).await?,
                Direction::SOUTH => self.move_relative(0, 0, -1).await?,
                Direction::WEST => self.move_relative(1, 0, 0).await?,
            };
            
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn up(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Up).await?;

        if result.success {
            self.move_relative(0, 1, 0).await?;
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn down(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Down).await?;

        if result.success {
            self.move_relative(0, -1, 0).await?;
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn turn_left(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::TurnLeft).await?;

        if result.success {
            match self.get_direction() {
                Direction::NORTH => self.rotate_direction(Direction::WEST).await?,
                Direction::EAST => self.rotate_direction(Direction::NORTH).await?,
                Direction::SOUTH => self.rotate_direction(Direction::EAST).await?,
                Direction::WEST => self.rotate_direction(Direction::SOUTH).await?,
            }
            
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn turn_right(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::TurnRight).await?;

        if result.success {
            match self.get_direction() {
                Direction::NORTH => self.rotate_direction(Direction::EAST).await?,
                Direction::EAST => self.rotate_direction(Direction::SOUTH).await?,
                Direction::SOUTH => self.rotate_direction(Direction::WEST).await?,
                Direction::WEST => self.rotate_direction(Direction::NORTH).await?,
            };
    
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn dig(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Dig{ side }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn dig_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::DigUp{ side }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn dig_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::DigDown{ side }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn place(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Place{ text }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn place_up(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::PlaceUp{ text }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn place_down(&mut self, text: Option<String>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::PlaceDown{ text }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn detect(&self) -> Result<bool, TurtleError> {
        let result = self.query(turtle::Detect).await?;
        Ok(result.unwrap_or(false))
    }

    async fn detect_up(&self) -> Result<bool, TurtleError> {
        let result = self.query(turtle::DetectUp).await?;
        Ok(result.unwrap_or(false))
    }

    async fn detect_down(&self) -> Result<bool, TurtleError> {
        let result = self.query(turtle::DetectDown).await?;
        Ok(result.unwrap_or(false))
    }

    async fn inspect(&self) -> Result<Option<Value>, TurtleError> {
        self.query(turtle::Inspect).await
    }

    async fn inspect_up(&self) -> Result<Option<Value>, TurtleError> {
        self.query(turtle::InspectUp).await
    }

    async fn inspect_down(&self) -> Result<Option<Value>, TurtleError> {
        self.query(turtle::InspectDown).await
    }

    async fn select(&mut self, slot: Slot) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Select { slot }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn get_selected_slot(&self) -> Result<Slot, TurtleError> {
        self.query(turtle::GetSelectedSlot).await
    }

    async fn get_item_count(&self, slot: Option<Slot>) -> Result<u8, TurtleError> {
        self.query(turtle::GetItemCount{ slot }).await
    }

    async fn get_item_space(&self, slot: Option<Slot>) -> Result<u8, TurtleError> {
        self.query(turtle::GetItemSpace{ slot }).await
    }

    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<Value>, TurtleError> {
        self.query(turtle::GetItemDetail{ slot, detailed }).await
    }

    async fn drop(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Drop{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::DropUp{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::DropDown{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn suck(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Suck{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::SuckUp{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::SuckDown{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::TransferTo { slot, count }).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn compare(&self) -> Result<bool, TurtleError> {
        self.query(turtle::Compare).await
    }

    async fn compare_up(&self) -> Result<bool, TurtleError> {
        self.query(turtle::CompareUp).await
    }

    async fn compare_down(&self) -> Result<bool, TurtleError> {
        self.query(turtle::CompareDown).await
    }

    async fn compare_to(&self, slot: Slot) -> Result<bool, TurtleError> {
        self.query(turtle::CompareTo { slot }).await
    }

    async fn get_fuel_level(&self) -> FuelLevel {
        self.query(turtle::GetFuelLevel).await.unwrap_or(FuelLevel::Amount(0))
    }

    async fn get_fuel_limit(&self) -> FuelLevel {
        self.query(turtle::GetFuelLimit).await.unwrap_or(FuelLevel::Amount(0))
    }

    async fn refuel(&mut self, count: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Refuel{ count }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn equip_left(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::EquipLeft).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn equip_right(&mut self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::EquipRight).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn get_equipped_left(&self) -> Result<Option<serde_json::Value>, TurtleError> {
        self.query(turtle::GetEquippedLeft).await
    }

    async fn get_equipped_right(&self) -> Result<Option<serde_json::Value>, TurtleError> {
        self.query(turtle::GetEquippedRight).await
    }

    async fn craft(&mut self, limit: Option<u8>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Craft{ limit }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn attack(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::Attack{ side }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn attack_up(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::AttackUp{ side }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }

    async fn attack_down(&mut self, side: Option<Side>) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::AttackDown{ side }).await?;
        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("No reason specified.".to_string())))
        }
    }
}

/// Smart turtle implementation
impl SmartTurtle for Turtle {
    // GPS functions
    async fn start_gps_host(&self) -> Result<(), TurtleError> {
        let result = self.execute(TurtleAction::StartGpsHost).await?;

        if result.success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(result.reason.unwrap_or("Error hosting GPS".to_string())))
        }
    }

    async fn stop_gps_host(&self) -> Result<(), TurtleError> {
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

        let forward = match self.inspect().await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        let down = match self.inspect_down().await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        let up = match self.inspect_up().await {
            Ok(Some(data)) => data["name"].as_str().unwrap_or("minecraft:air").to_string(),
            _ => "minecraft:air".to_string(),
        };

        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(fx, fy, fz, forward.clone())}).await.unwrap();
        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(x, y + 1, z, up.clone())}).await.unwrap();
        server_tx.send(ServerMessage::Oneshot { client_id: self.get_id(), action: ServerAction::UpdateBlock(x, y - 1, z, down.clone())}).await.unwrap();

        Ok((forward.to_string(), up.to_string(), down.to_string()))
    }
    
    // Movement functions
    async fn face_block(&mut self, x: i64, z: i64) -> Result<(), TurtleError> {
        // Get the current direction
        let current_direction = self.get_direction() as i8;
        
        // Convert x and z offsets to a target direction
        let new_direction = match (x.signum(), z.signum()) {
            (1, 0) => Direction::EAST,
            (-1, 0) => Direction::WEST,
            (0, -1) => Direction::NORTH,
            (0, 1) => Direction::SOUTH,
            (0, 0) => return Ok(()), // No offset, no need to turn
            _ => return Err(TurtleError::VirtualError("Non-cardinal offset".to_string()))
        } as i8;

        // Calculate the number of 90-degree right turns needed
        let right_turns = (new_direction - current_direction).rem_euclid(4);

        // Execute the most efficient turns
        match right_turns {
            1 => {
                self.turn_right().await?;
            }
            2 => {
                self.turn_right().await?;
                self.turn_right().await?;
            }
            3 => {
                self.turn_left().await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn move_to(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError> {
        // Moves to a spot without pathfinding, first elevation, then x, then z
        for _ in 0..dy.abs() {
            if dy > 0 {
                self.up().await?;
                self.scan_blocks().await?;
            } else {
                self.down().await?;
                self.scan_blocks().await?;
            }
        }

        self.face_block(dx, 0).await?;
        self.scan_blocks().await?;

        for _ in 0..dx.abs() {
            self.forward().await?;
            self.scan_blocks().await?;
        }

        self.face_block(0, dz).await?;
        self.scan_blocks().await?;

        for _ in 0..dz.abs() {
            self.forward().await?;
            self.scan_blocks().await?;
        }

        Ok(())
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
            let mut try_again = false;
            let last_spot = path.pop().unwrap_or(Vector3::new(dest_x, dest_y, dest_z));

            for next in &path {
                let pos = Vector3::from(self.get_position());
                let delta = *next - pos;

                if let Err(e) = self.move_to(delta.x, delta.y, delta.z).await {
                    // Something went wrong on the path to it, restart the loop and try again
                    match e {
                        TurtleError::VirtualError(e) => {
                            eprintln!("Non-viable path {e}");
                            try_again = true;
                        },
                        TurtleError::SocketError(_) => return Err(e),
                    }

                    break;
                }
            }

            if try_again {
                continue;
            }

            // No more path was blocked, we must be at the end
            if skip_last {
                // Just face towards it
                let pos = Vector3::from(self.get_position());
                let delta = last_spot - pos;
                self.face_block(delta.x, delta.z).await?;
            } else {
                // Otherwise, move to it
                let pos = Vector3::from(self.get_position());
                let delta = last_spot - pos;
                self.move_to(delta.x, delta.y, delta.z).await?;
            }

            break;
        }

        Ok(())
    }
}
