/// This module is responsible for managing blocks in the world, including updating block states and notifying visualizers of changes.
use crate::managers::object_relations::ORM;
use shared::blocks::{Block, BlockNotification};
use tokio::sync;

/// High speed in-memory cache


/// Common interactor for outside module
pub struct BlockManager {
    database: ORM,
    notifications_tx: sync::broadcast::Sender<BlockNotification>,
    notifications_rx: sync::broadcast::Receiver<BlockNotification>
}

impl BlockManager {
    pub async fn new() -> Self {
        let (tx, rx) = sync::broadcast::channel::<BlockNotification>(2048);

        BlockManager {
            database: ORM::new().await,
            notifications_tx: tx,
            notifications_rx: rx
        }
    }

    pub fn subscribe(&self) -> sync::broadcast::Receiver<BlockNotification> {
        self.notifications_tx.subscribe()
    }

    pub async fn update_block(&mut self, x: i64, y: i64, z: i64, block_type: String) -> Result<(), String> {
        let block = Block {
            x,
            y,
            z,
            block_type,
        };

        self.notifications_tx.send(BlockNotification::Update(block.clone())).unwrap();
        self.database.upsert_block(block).await.map_err(|e| e.to_string())
    }

    pub async fn get_block(&self, x: i64, y: i64, z: i64) -> Option<Block> {
        self.database.get_block(x, y, z).await
    }

    pub async fn remove_block(&mut self, x: i64, y: i64, z: i64) -> Result<(), String> {
        self.notifications_tx.send(BlockNotification::Remove(x, y, z)).unwrap();
        self.database.remove_block(x, y, z).await.map_err(|e| e.to_string())
    }

    pub async fn get_all_blocks(&self) -> Vec<Block> {
        self.database.get_all_blocks().await.unwrap()
    }
}