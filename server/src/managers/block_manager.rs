use std::rc::Rc;

/// This module is responsible for managing blocks in the world, including updating block states and notifying visualizers of changes.
use crate::object_relations::ORM;
use shared::blocks::{Block, BlockNotification};

pub struct BlockManager {
    database: Rc<ORM>,
    notifications: Vec<BlockNotification>,
}

impl BlockManager {
    pub fn new(database: Rc<ORM>) -> Self {
        BlockManager { database, notifications: Vec::new() }
    }

    pub fn update_block(&mut self, x: i64, y: i64, z: i64, block_type: String) -> Result<(), String> {
        let block = Block {
            x,
            y,
            z,
            block_type,
        };

        self.notifications.push(BlockNotification::Update(block.clone()));
        self.database.upsert_block(&block).map_err(|e| e.to_string())
    }

    pub fn get_block(&self, x: i64, y: i64, z: i64) -> Option<Block> {
        self.database.get_block(x, y, z)
    }

    pub fn remove_block(&mut self, x: i64, y: i64, z: i64) -> Result<(), String> {
        self.notifications.push(BlockNotification::Remove(x, y, z));
        self.database.remove_block(x, y, z).map_err(|e| e.to_string())
    }

    pub fn get_all_blocks(&self) -> Vec<Block> {
        self.database.get_all_blocks().unwrap()
    }

    pub fn is_notification_pending(&self) -> bool {
        !self.notifications.is_empty()
    }

    pub fn pop_notification(&mut self) -> BlockNotification {
        self.notifications.remove(0)
    }
}