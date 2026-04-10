/// This module is responsible for managing blocks in the world, including updating block states and notifying visualizers of changes.
use std::sync::Arc;

use crate::managers::object_relations::ORM;
use shared::blocks::{Block, BlockNotification};
use tokio::sync::Mutex;

pub struct BlockManager {
    database: Arc<Mutex<ORM>>,
    notifications: Vec<BlockNotification>,
}

impl BlockManager {
    pub fn new(database: Arc<Mutex<ORM>>) -> Self {
        BlockManager { database, notifications: Vec::new() }
    }

    pub async fn update_block(&mut self, x: i64, y: i64, z: i64, block_type: String) -> Result<(), String> {
        let block = Block {
            x,
            y,
            z,
            block_type,
        };

        self.notifications.push(BlockNotification::Update(block.clone()));
        self.database.lock().await.upsert_block(&block).map_err(|e| e.to_string())
    }

    pub async fn get_block(&self, x: i64, y: i64, z: i64) -> Option<Block> {
        self.database.lock().await.get_block(x, y, z)
    }

    pub async fn remove_block(&mut self, x: i64, y: i64, z: i64) -> Result<(), String> {
        self.notifications.push(BlockNotification::Remove(x, y, z));
        self.database.lock().await.remove_block(x, y, z).map_err(|e| e.to_string())
    }

    pub async fn get_all_blocks(&self) -> Vec<Block> {
        self.database.lock().await.get_all_blocks().unwrap()
    }

    pub async fn is_notification_pending(&self) -> bool {
        !self.notifications.is_empty()
    }

    pub async fn pop_notification(&mut self) -> BlockNotification {
        self.notifications.remove(0)
    }
}