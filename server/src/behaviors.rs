use crate::AppState;

pub trait TurtleMessage {
    fn handle_message(&self, client_id: u64, app_state: &mut AppState) -> Result<(), String>;
}

pub struct BlockUpdate {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub block_type: String,
}
impl TurtleMessage for BlockUpdate {
    fn handle_message(&self, _: u64, app_state: &mut AppState) -> Result<(), String> {
        app_state.block_manager.update_block(self.x, self.y, self.z, self.block_type.clone())?;
        Ok(())
    }
}

pub struct PositionUpdate {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
impl TurtleMessage for PositionUpdate {
    fn handle_message(&self, client_id: u64, app_state: &mut AppState) -> Result<(), String> {
        let turtle = app_state.client_manager.get_turtle(client_id).unwrap();
        turtle.update_spatial(self.x, self.y, self.z, turtle.get_rotation());
        Ok(())
    }
}

pub struct RotationUpdate {
    pub rotation: i8,
}
impl TurtleMessage for RotationUpdate {
    fn handle_message(&self, client_id: u64, app_state: &mut AppState) -> Result<(), String> {
        let turtle = app_state.client_manager.get_turtle(client_id).unwrap();
        turtle.update_spatial(turtle.get_position().0, turtle.get_position().1, turtle.get_position().2, self.rotation);
        Ok(())
    }
}