use std::sync::Arc;

use futures_util::lock::Mutex;

use crate::turtle::Turtle;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TaskAction {
    MoveTo{ x: i64, y: i64, z: i64 },
    Craft{ items: [Option<String>; 9] },
    Place{ x: i64, y: i64, z: i64, block: String },
    Break{ x: i64, y: i64, z: i64 },
    Drop{ x: i64, y: i64, z: i64, item: String },
    Suck{ x: i64, y: i64, z: i64 },
    Chain{ list: Vec<TaskAction> }
}