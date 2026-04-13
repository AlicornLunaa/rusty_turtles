use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::turtle::client::Turtle;

#[derive(Clone)]
pub struct TurtleManager {
    turtles: Arc<RwLock<HashMap<u64, Arc<Mutex<Turtle>>>>>,
    next_id: Arc<RwLock<u64>>,
}

impl TurtleManager {
    pub fn new() -> Self {
        Self {
            turtles: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_next_id(&self) -> u64 {
        *self.next_id.read().await
    }

    pub async fn add_turtle(&mut self, turtle: Arc<Mutex<Turtle>>) -> u64 {
        let id = self.next_id.read().await;
        self.turtles.write().await.insert(*id, turtle);
        *self.next_id.write().await += 1;
        *id
    }

    pub async fn remove_turtle(&mut self, id: u64) -> bool {
        self.turtles.write().await.remove(&id).is_some()
    }

    pub async fn get_turtle(&self, id: u64) -> Option<Arc<Mutex<Turtle>>> {
        self.turtles.read().await.get(&id).cloned()
    }

    pub async fn iter_turtles(&self) -> Vec<Arc<Mutex<Turtle>>> {
        self.turtles.read().await.values().cloned().collect()
    }
}