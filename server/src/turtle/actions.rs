use serde_json::{Value, json};

use crate::turtle::{SmartTurtle, client::Turtle, traits::VirtualTurtle, types::{Direction, FuelLevel, Side, Slot, TurtleError}};

/// Virtual turtle implementation for turtle
impl VirtualTurtle for Turtle {
    async fn forward(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "forward", "args": [] })).await?;

        match self.get_direction() {
            Direction::NORTH => self.move_relative(0, 0, -1).await?,
            Direction::EAST => self.move_relative(1, 0, 0).await?,
            Direction::SOUTH => self.move_relative(0, 0, 1).await?,
            Direction::WEST => self.move_relative(-1, 0, 0).await?,
        };

        Ok(())
    }

    async fn back(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "back", "args": [] })).await?;

        match self.get_direction() {
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

        match self.get_direction() {
            Direction::NORTH => self.rotate_direction(Direction::WEST).await?,
            Direction::EAST => self.rotate_direction(Direction::NORTH).await?,
            Direction::SOUTH => self.rotate_direction(Direction::EAST).await?,
            Direction::WEST => self.rotate_direction(Direction::SOUTH).await?,
        }
        
        Ok(())
    }

    async fn turn_right(&mut self) -> Result<(), TurtleError> {
        self.rpc_success_error(json!({ "action": "turnRight", "args": [] })).await?;

        match self.get_direction() {
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

/// Smart turtle implementation
impl SmartTurtle for Turtle {
    // GPS functions
    async fn start_gps_host(&self) -> Result<(), TurtleError> {
        let (x, y, z) = self.get_position();
        let result = self.remote_procedure_call(json!({ "action": "start_gps_host", "args": [x, y, z] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError("Error hosting GPS".to_string()))
        }
    }

    async fn stop_gps_host(&self) -> Result<(), TurtleError> {
        let result = self.remote_procedure_call(json!({ "action": "stop_gps_host", "args": [] })).await?;
        let success = result["success"].as_bool().unwrap_or(false);
        let reason = result["error"].as_str();

        if success {
            Ok(())
        } else {
            Err(TurtleError::VirtualError(reason.unwrap_or("Unspecified error").to_string()))
        }
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
            } else {
                self.down().await?;
            }
        }

        self.face_block(dx, 0).await?;

        for _ in 0..dx.abs() {
            self.forward().await?;
        }

        self.face_block(0, dz).await?;

        for _ in 0..dz.abs() {
            self.forward().await?;
        }

        Ok(())
    }

    async fn path_to(&mut self, x: i64, y: i64, z: i64) -> Result<(), TurtleError> {
        // Pathfinds to a location
        todo!()
    }
}