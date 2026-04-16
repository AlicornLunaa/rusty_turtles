use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use tokio::sync::{Mutex, RwLock};

use crate::turtle::client::Turtle;

#[derive(Clone)]
pub struct TurtleManager {
    turtles: Arc<RwLock<HashMap<u64, Arc<Mutex<Turtle>>>>>,
    next_id: Arc<AtomicU64>,
}

impl TurtleManager {
    pub fn new() -> Self {
        Self {
            turtles: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get_next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn add_turtle(&self, turtle: Arc<Mutex<Turtle>>) {
        let id = turtle.lock().await.get_id();
        self.turtles.write().await.insert(id, turtle);
    }

    pub async fn remove_turtle(&self, id: u64) -> bool {
        self.turtles.write().await.remove(&id).is_some()
    }

    pub async fn get_turtle(&self, id: u64) -> Option<Arc<Mutex<Turtle>>> {
        self.turtles.read().await.get(&id).cloned()
    }

    pub async fn iter_turtles(&self) -> Vec<Arc<Mutex<Turtle>>> {
        self.turtles.read().await.values().cloned().collect()
    }
}