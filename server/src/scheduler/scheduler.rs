use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};

use tokio::sync::{Mutex, Notify};

use crate::scheduler::actions::TaskAction;

/// Job object
#[derive(Clone, Eq, PartialEq, Debug)]
struct Task {
    priority: i64,
    action: TaskAction,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Job Scheduler
#[derive(Clone)]
pub struct TaskScheduler {
    tasks: Arc<Mutex<BinaryHeap<Task>>>,
    notify_new: Arc<Notify>
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            notify_new: Arc::new(Notify::new())
        }
    }

    pub async fn add_task(&self, action: TaskAction, priority: i64) {
        self.tasks.lock().await.push(Task { priority, action });
        self.notify_new.notify_one();
    }

    pub async fn pop_task(&self) -> Option<TaskAction> {
        match self.tasks.lock().await.pop() {
            Some(job) => Some(job.action),
            None => None,
        }
    }

    pub async fn len(&self) -> usize {
        self.tasks.lock().await.len()
    }

    pub async fn wait_for_notif(&self){
        self.notify_new.notified().await
    }
}