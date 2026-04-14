use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use tokio::sync::{Mutex, RwLock};

use crate::client::client::Client;

#[derive(Clone)]
pub struct ClientManager {
    clients: Arc<RwLock<HashMap<u64, Arc<Mutex<Client>>>>>,
    next_id: Arc<AtomicU64>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get_next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn add_client(&self, client: Arc<Mutex<Client>>) -> u64 {
        let id = self.next_id.load(Ordering::SeqCst);
        self.clients.write().await.insert(id, client);
        id
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