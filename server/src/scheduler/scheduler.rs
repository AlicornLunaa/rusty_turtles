use std::{cmp::Ordering, collections::BinaryHeap};

use crate::scheduler::actions::JobAction;

/// Job object
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Job {
    priority: i64,
    action: JobAction,
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


/// Job Scheduler
pub struct JobScheduler {
    jobs: BinaryHeap<Job>,
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {
            jobs: BinaryHeap::new()
        }
    }

    pub fn add_job(&mut self, action: JobAction, priority: i64) {
        self.jobs.push(Job { priority, action });
    }

    pub fn pop_job(&mut self) -> Option<JobAction> {
        match self.jobs.pop() {
            Some(job) => Some(job.action),
            None => None,
        }
    }

    pub fn queue_size(&self) -> usize {
        self.jobs.len()
    }
}