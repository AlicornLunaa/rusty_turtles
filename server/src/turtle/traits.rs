use crate::turtle::types::TurtleError;

/// Trait for turtles with more than 
pub trait SmartTurtle {
    // GPS functions
    async fn start_gps_host(&mut self) -> Result<(), TurtleError>;
    async fn stop_gps_host(&mut self) -> Result<(), TurtleError>;

    // Auto scanners
    async fn scan_blocks(&self) -> Result<(String, String, String), TurtleError>;

    // Smart movement
    async fn move_to(&mut self, dx: i64, dy: i64, dz: i64) -> Result<(), TurtleError>;
    async fn path_to(&mut self, x: i64, y: i64, z: i64, skip_last: bool) -> Result<(), TurtleError>;
}