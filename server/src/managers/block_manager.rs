use std::sync::Arc;

/// This module is responsible for managing blocks in the world, including updating block states and notifying visualizers of changes.
use crate::managers::object_relations::ORM;
use dashmap::DashMap;
use shared::blocks::{Block, BlockNotification};
use tokio::sync::{self, broadcast, mpsc};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct Coord { x: i64, y: i64, z: i64 }

/// Interactions
pub enum BlockManagerCommand {
    UpdateBlock { x: i64, y: i64, z: i64, block_type: String },
    RemoveBlock { x: i64, y: i64, z: i64 },
    GetAllBlocks { reply: sync::oneshot::Sender<Vec<Block>> }
}

/// Common interactor for outside module
#[derive(Clone)]
pub struct BlockManager {
    grid: Arc<DashMap<Coord, String>>,
    database_tx: mpsc::Sender<BlockManagerCommand>,
    notification_tx: sync::broadcast::Sender<BlockNotification>,
}

impl BlockManager {
    pub async fn new() -> Self {
        let grid = Arc::new(DashMap::new());
        let (database_tx, mut database_rx) = mpsc::channel(1000);
        let (notification_tx, _) = broadcast::channel(100);

        tokio::spawn(async move {
            let db_connection = ORM::new().await;
            
            while let Some(cmd) = database_rx.recv().await {
                match cmd {
                    BlockManagerCommand::UpdateBlock { x, y, z, block_type } => {
                        // Await the slow database insert here
                        db_connection.upsert_block(Block { x, y, z, block_type }).await.unwrap();
                    },
                    BlockManagerCommand::RemoveBlock { x, y, z } => {
                        db_connection.remove_block(x, y, z).await.unwrap();
                    },
                    BlockManagerCommand::GetAllBlocks { reply } => {
                        let block_list = db_connection.get_all_blocks().await;
                        reply.send(block_list.unwrap_or(Vec::new())).unwrap();
                    },
                }
            }
        });

        // Read the entirety of the database into the grid
        let (tx, rx) = sync::oneshot::channel();
        database_tx.send(BlockManagerCommand::GetAllBlocks { reply: tx }).await.unwrap();

        let block_data = match rx.await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Block manager error. {e}\nPersistence probably does not work...");
                Vec::new()
            },
        };

        for block in block_data {
            grid.insert(Coord { x: block.x, y: block.y, z: block.z }, block.block_type.clone());
        }

        // Return the completed object
        BlockManager {
            grid,
            database_tx,
            notification_tx,
        }
    }

    pub fn subscribe(&self) -> sync::broadcast::Receiver<BlockNotification> {
        self.notification_tx.subscribe()
    }

    pub async fn update_block(&self, x: i64, y: i64, z: i64, block_type: String) {
        self.grid.insert(Coord { x, y, z }, block_type.clone());

        let _ = self.notification_tx.send(BlockNotification::Update(Block { x, y, z, block_type: block_type.clone() }));
        let _ = self.database_tx.send(BlockManagerCommand::UpdateBlock { x, y, z, block_type }).await;
    }

    pub async fn remove_block(&self, x: i64, y: i64, z: i64) {
        self.grid.remove(&Coord { x, y, z });

        let _ = self.notification_tx.send(BlockNotification::Remove(x, y, z ));
        let _ = self.database_tx.send(BlockManagerCommand::RemoveBlock { x, y, z }).await;
    }

    pub fn get_block(&self, x: i64, y: i64, z: i64) -> Option<String> {
        self.grid.get(&Coord { x, y, z }).map(|entry| entry.value().clone())
    }

    pub fn iter_blocks(&self) -> Vec<Block> {
        self.grid.iter().map(|entry| {
            let coord = entry.key();
            let block_type = entry.value().clone();
            Block { x: coord.x, y: coord.y, z: coord.z, block_type }
        }).collect()
    }
}