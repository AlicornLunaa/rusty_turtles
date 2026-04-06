use std::collections::HashMap;
use simple_websockets::{Message, Responder};
use crate::{agent::AgentType, turtle::Turtle};

/// This structure is responsible for holding each different kind of client
pub struct ClientManager {
    waiting_room: HashMap<u64, Responder>,
    turtle_clients: HashMap<u64, Turtle>,
    visualizer_clients: HashMap<u64, Responder>,
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            waiting_room: HashMap::new(),
            turtle_clients: HashMap::new(),
            visualizer_clients: HashMap::new(),
        }
    }

    pub fn get_responder(&self, client_id: u64) -> Option<&Responder> {
        self.waiting_room.get(&client_id)
            .or_else(|| self.turtle_clients.get(&client_id).map(|turtle| &turtle.responder))
            .or_else(|| self.visualizer_clients.get(&client_id))
    }

    pub fn send(&self, client_id: u64, message: Message) -> Result<(), String> {
        if let Some(responder) = self.get_responder(client_id) {
                responder.send(message);
                Ok(())
            } else {
                Err(format!("Client #{} not found", client_id))
            }
    }

    pub fn add_to_waiting_room(&mut self, client_id: u64, responder: Responder) {
        self.waiting_room.insert(client_id, responder);
    }

    pub fn qualify_agent(&mut self, client_id: u64, agent_type: AgentType) -> Result<(), String> {
        if let Some(responder) = self.waiting_room.remove(&client_id) {
            match agent_type {
                AgentType::Turtle => {
                    self.turtle_clients.insert(client_id, Turtle::new(responder, client_id));
                },
                AgentType::Client => {
                    self.visualizer_clients.insert(client_id, responder);
                },
            }

            Ok(())
        } else {
            Err("Client not found in waiting room".into())
        }
    }

    pub fn remove_client(&mut self, client_id: u64) {
        println!("Client #{} disconnected.", client_id);

        if let Some(responder) = self.get_responder(client_id) {
            responder.close();
        }

        self.waiting_room.remove(&client_id);
        self.turtle_clients.remove(&client_id);
        self.visualizer_clients.remove(&client_id);
    }

    pub fn get_turtle(&mut self, client_id: u64) -> Option<&mut Turtle> {
        self.turtle_clients.get_mut(&client_id)
    }

    pub fn get_turtle_clients(&self) -> &HashMap<u64, Turtle> {
        &self.turtle_clients
    }

    pub fn get_visualizer_clients(&self) -> &HashMap<u64, Responder> {
        &self.visualizer_clients
    }

    pub fn get_agent_type(&self, client_id: u64) -> Option<AgentType> {
        if self.turtle_clients.contains_key(&client_id) {
            Some(AgentType::Turtle)
        } else if self.visualizer_clients.contains_key(&client_id) {
            Some(AgentType::Client)
        } else {
            None
        }
    }

    pub fn is_waiting(&self, client_id: u64) -> bool {
        self.waiting_room.contains_key(&client_id)
    }
}