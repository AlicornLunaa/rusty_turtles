#[derive(Debug)]
pub enum AgentType {
    Turtle, // Computercraft Turtle
    Client, // Visualizer program
}

#[derive(Debug)]
pub enum TurtleMessage {
    PositionUpdate { x: i64, y: i64, z: i64 },
    RotationUpdate { rotation: i8 },
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