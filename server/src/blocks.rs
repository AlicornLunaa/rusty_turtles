/// This file contains the custom data types used in the server, such as messages, blocks, and other game-related structures.
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_type: String,
}

pub struct Chest {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub count: u16, // The quantity of the item stored in the chest
    pub max_count: u16, // The maximum capacity of the chest
    pub item_type: String, // The type of item stored in the chest
}