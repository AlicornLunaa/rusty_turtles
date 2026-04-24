use std::sync::Arc;

use crate::managers::{object_relations::ORM, turtle_manager::TurtleManager};
use dashmap::DashMap;
use shared::blocks::{Chest, ChestNotification};
use tokio::sync::{self, broadcast, mpsc};

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct Coord { x: i64, y: i64, z: i64 }

enum DatabaseCommand {
    UpdateInventory { chest: Chest },
    RemoveInventory { x: i64, y: i64, z: i64 },
    GetAllChests { reply: sync::oneshot::Sender<Vec<Chest>> }
}

/// Common interactor for outside module
#[derive(Clone)]
pub struct InventoryManager {
    grid: Arc<DashMap<Coord, (String, u16, u16)>>,
    turtle_manager: TurtleManager,
    database_tx: mpsc::Sender<DatabaseCommand>,
    notification_tx: sync::broadcast::Sender<ChestNotification>,
}

impl InventoryManager {
    pub async fn new(turtle_manager: TurtleManager) -> Self {
        let grid = Arc::new(DashMap::new());
        let (database_tx, mut database_rx) = mpsc::channel(1000);
        let (notification_tx, _) = broadcast::channel(100);

        tokio::spawn(async move {
            let db_connection = ORM::new().await;
            
            while let Some(cmd) = database_rx.recv().await {
                match cmd {
                    DatabaseCommand::UpdateInventory { chest } => {
                        // Await the slow database insert here
                        db_connection.upsert_chest(&chest).await.unwrap();
                    },
                    DatabaseCommand::RemoveInventory { x, y, z } => {
                        db_connection.remove_chest(x, y, z).await.unwrap();
                    },
                    DatabaseCommand::GetAllChests { reply } => {
                        let chest_list = db_connection.get_all_chests().await;
                        reply.send(chest_list.unwrap_or(Vec::new())).unwrap();
                    },
                }
            }
        });

        // Read the entirety of the database into the grid
        let (tx, rx) = sync::oneshot::channel();
        database_tx.send(DatabaseCommand::GetAllChests { reply: tx }).await.unwrap();

        let block_data = match rx.await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Block manager error. {e}\nPersistence probably does not work...");
                Vec::new()
            },
        };

        for block in block_data {
            grid.insert(Coord { x: block.x, y: block.y, z: block.z }, (block.item_type, block.count, block.max_count));
        }

        // Return the completed object
        InventoryManager {
            grid,
            turtle_manager,
            database_tx,
            notification_tx,
        }
    }

    pub fn subscribe(&self) -> sync::broadcast::Receiver<ChestNotification> {
        self.notification_tx.subscribe()
    }

    pub async fn update_chest(&self, x: i64, y: i64, z: i64, item_type: String, count: u16, max_count: u16) {
        self.grid.insert(Coord { x, y, z }, (item_type.clone(), count, max_count));

        let chest = Chest { x, y, z, item_type, count, max_count };
        let _ = self.notification_tx.send(ChestNotification::Update(chest.clone()));
        let _ = self.database_tx.send(DatabaseCommand::UpdateInventory { chest }).await;
    }

    pub async fn remove_chest(&self, x: i64, y: i64, z: i64) {
        self.grid.remove(&Coord { x, y, z });

        let _ = self.notification_tx.send(ChestNotification::Remove(x, y, z ));
        let _ = self.database_tx.send(DatabaseCommand::RemoveInventory { x, y, z }).await;
    }

    pub fn get_chest(&self, x: i64, y: i64, z: i64) -> Option<Chest> {
        self.grid.get(&Coord { x, y, z }).map(|entry| {
            let (item_type, count, max_count) = entry.value().clone();
            Chest { x, y, z, item_type, count, max_count }
        })
    }

    pub fn iter_chests(&self) -> Vec<Chest> {
        self.grid.iter().map(|entry| {
            let coord = entry.key();
            let (item_type, count, max_count) = entry.value().clone();
            Chest { x: coord.x, y: coord.y, z: coord.z, item_type, count, max_count }
        }).collect()
    }
}