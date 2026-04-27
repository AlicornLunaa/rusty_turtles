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
}

fn move_to(turtle: &Arc<Mutex<Turtle>>, x: i64, y: i64, z: i64) -> bool {
    todo!()
}

pub async fn dispatch(turtle: &Arc<Mutex<Turtle>>, task: TaskAction) -> bool {
    match task {
        TaskAction::MoveTo { x, y, z } => move_to(turtle, x, y, z),
        TaskAction::Craft { items } => todo!(),
        TaskAction::Place { x, y, z, block } => todo!(),
        TaskAction::Break { x, y, z } => todo!(),
        TaskAction::Drop { x, y, z, item } => todo!(),
        TaskAction::Suck { x, y, z } => todo!(),
    }
}