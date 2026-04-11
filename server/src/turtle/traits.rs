use serde_json::Value;

use crate::turtle::types::{FuelLevel, Side, Slot, TurtleError};

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
    async fn inspect(&self) -> Result<Option<Value>, TurtleError>;
    async fn inspect_up(&self) -> Result<Option<Value>, TurtleError>;
    async fn inspect_down(&self) -> Result<Option<Value>, TurtleError>;

    // Inventory Management
    async fn select(&mut self, slot: Slot) -> Result<(), TurtleError>;
    async fn get_selected_slot(&self) -> Result<Slot, TurtleError>;
    async fn get_item_count(&self, slot: Option<Slot>) -> Result<u8, TurtleError>;
    async fn get_item_space(&self, slot: Option<Slot>) -> Result<u8, TurtleError>;
    async fn get_item_detail(&self, slot: Option<Slot>, detailed: Option<bool>) -> Result<Option<Value>, TurtleError>;
    async fn drop(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn drop_up(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn drop_down(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck_up(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn suck_down(&mut self, count: Option<u8>) -> Result<(), TurtleError>;
    async fn transfer_to(&mut self, slot: Slot, count: Option<u8>) -> Result<(), TurtleError>;
    async fn compare(&self) -> Result<bool, TurtleError>;
    async fn compare_up(&self) -> Result<bool, TurtleError>;
    async fn compare_down(&self) -> Result<bool, TurtleError>;
    async fn compare_to(&self, slot: Slot) -> Result<bool, TurtleError>;

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

/// Trait for turtles with more than 
pub trait SmartTurtle {
    // GPS functions
    async fn start_gps_host(&self) -> Result<(), TurtleError>;
    async fn stop_gps_host(&self) -> Result<(), TurtleError>;

    // Auto scanners
    async fn scan_blocks(&self) -> Result<(String, String, String), TurtleError>;

    // Smart movement
    async fn face_block(&mut self, x: i64, z: i64) -> Result<(), TurtleError>;
    async fn move_to(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError>;
    async fn path_to(&mut self, x: i64, y: i64, z: i64, skip_last: bool) -> Result<(), TurtleError>;
}