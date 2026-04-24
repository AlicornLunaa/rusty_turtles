use serde::{Deserialize, Serialize};

/// This file contains the custom data types used in the server, such as messages, blocks, and other game-related structures.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub block_type: String,
    pub last_updated: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockNotification {
    Update(Block),
    Remove(i64, i64, i64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chest {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub count: u16, // The quantity of the item stored in the chest
    pub max_count: u16, // The maximum capacity of the chest
    pub item_type: String, // The type of item stored in the chest
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChestNotification {
    Update(Chest),
    Remove(i64, i64, i64),
}