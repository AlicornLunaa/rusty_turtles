use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    UpdateBlock { x: i64, y: i64, z: i64, block_type: String }
}
