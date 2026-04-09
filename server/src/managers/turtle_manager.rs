use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::turtle::Turtle;

pub struct TurtleManager {
    turtles: HashMap<u64, Arc<Mutex<Turtle>>>,
    next_id: u64,
}

impl TurtleManager {
    pub fn new() -> Self {
        Self {
            turtles: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_turtle(&mut self, turtle: Arc<Mutex<Turtle>>) -> u64 {
        let id = self.next_id;
        self.turtles.insert(id, turtle);
        self.next_id += 1;
        id
    }

    pub fn remove_turtle(&mut self, id: u64) -> bool {
        self.turtles.remove(&id).is_some()
    }

    pub fn get_turtle(&self, id: u64) -> Option<&Arc<Mutex<Turtle>>> {
        self.turtles.get(&id)
    }

    pub fn get_turtle_mut(&mut self, id: u64) -> Option<&mut Arc<Mutex<Turtle>>> {
        self.turtles.get_mut(&id)
    }

    pub fn iter_turtles(&self) -> impl Iterator<Item = (&u64, &Arc<Mutex<Turtle>>)> {
        self.turtles.iter()
    }

    pub fn iter_turtles_mut(&mut self) -> impl Iterator<Item = (&u64, &mut Arc<Mutex<Turtle>>)> {
        self.turtles.iter_mut()
    }

    pub fn get_next_id(&self) -> u64 {
        self.next_id
    }
}