use crate::{client_manager::ClientManager, object_relations::ORM, turtle::Turtle};

#[derive(Debug)]
pub enum AgentType {
    Turtle, // Computercraft Turtle
    Client, // Visualizer program
}

pub trait TurtleMessage {
    fn handle_message(&self, client_id: u64, client_manager: &mut ClientManager, database: &ORM) -> Result<(), String>;
}
pub struct PositionUpdate {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
pub struct RotationUpdate {
    pub rotation: i8,
}
impl TurtleMessage for PositionUpdate {
    fn handle_message(&self, client_id: u64, client_manager: &mut ClientManager, database: &ORM) -> Result<(), String> {
        let turtle = client_manager.get_turtle(client_id).unwrap();
        turtle.update_spatial(self.x, self.y, self.z, turtle.get_rotation());
        Ok(())
    }
}
impl TurtleMessage for RotationUpdate {
    fn handle_message(&self, client_id: u64, client_manager: &mut ClientManager, database: &ORM) -> Result<(), String> {
        let turtle = client_manager.get_turtle(client_id).unwrap();
        turtle.update_spatial(turtle.get_position().0, turtle.get_position().1, turtle.get_position().2, self.rotation);
        Ok(())
    }
}

#[derive(Debug)]
pub enum TurtleOpCode {
    UpdatePosition = 0,
    UpdateRotation = 1,
}

impl TryFrom<u64> for TurtleOpCode {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TurtleOpCode::UpdatePosition),
            1 => Ok(TurtleOpCode::UpdateRotation),
            _ => Err(()),
        }
    }
}