use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::{client::Client};

#[derive(Clone)]
pub struct ClientManager {
    clients: Arc<RwLock<HashMap<u64, Arc<Mutex<Client>>>>>,
    next_id: Arc<RwLock<u64>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_next_id(&self) -> u64 {
        *self.next_id.read().await
    }

    pub async fn add_client(&self, turtle: Arc<Mutex<Client>>) -> u64 {
        let id = self.next_id.read().await;
        self.clients.write().await.insert(*id, turtle);
        *self.next_id.write().await += 1;
        *id
    }

    pub async fn remove_client(&self, id: u64) -> bool {
        self.clients.write().await.remove(&id).is_some()
    }

    pub async fn get_client(&self, id: u64) -> Option<Arc<Mutex<Client>>> {
        self.clients.read().await.get(&id).cloned()
    }

    pub async fn iter_clients(&self) -> Vec<Arc<Mutex<Client>>> {
        self.clients.read().await.values().cloned().collect()
    }
}