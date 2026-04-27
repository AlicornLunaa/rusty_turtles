use std::{cmp::Ordering, collections::BinaryHeap};

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
pub struct TaskScheduler {
    tasks: BinaryHeap<Task>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: BinaryHeap::new()
        }
    }

    pub fn add_task(&mut self, action: TaskAction, priority: i64) {
        self.tasks.push(Task { priority, action });
    }

    pub fn pop_task(&mut self) -> Option<TaskAction> {
        match self.tasks.pop() {
            Some(job) => Some(job.action),
            None => None,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}