use serde::{Deserialize, Serialize};

// Opcodes for turtles, this one needs numbers because it's sent to a lua system
#[derive(Debug)]
pub enum TurtleOpCode {
    UpdatePosition = 0,
    UpdateRotation = 1,
    BlockUpdate = 2,
}

impl TryFrom<u64> for TurtleOpCode {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TurtleOpCode::UpdatePosition),
            1 => Ok(TurtleOpCode::UpdateRotation),
            2 => Ok(TurtleOpCode::BlockUpdate),
            _ => Err(()),
        }
    }
}

// Opcodes for clients
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientOpCode {
    GetBlocks,
}