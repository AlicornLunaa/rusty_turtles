use dashmap::DashMap;
use tokio::task::JoinHandle;

use crate::{managers::{path_manager::AsyncPathManager, turtle_manager::{self, TurtleManager}}, scheduler::TaskScheduler};

pub struct SchedulerSystem {
    scheduler: TaskScheduler,
    turtles: TurtleManager,
    paths: AsyncPathManager,
    active_turtles: DashMap<u64, JoinHandle<()>>
}

impl SchedulerSystem {
    pub fn new(scheduler: TaskScheduler, turtle_manager: TurtleManager, paths: AsyncPathManager) -> Self {
        Self {
            scheduler,
            turtles: turtle_manager,
            paths,
            active_turtles: DashMap::new()
        }
    }

    pub fn start_turtle(&mut self, id: u64){
        // Starts the thread for controlling this one turtle
        let scheduler = self.scheduler.clone();

        let handle = tokio::spawn(async move {
            // This is the control loop thread, it will start the turtle runtime execution
            // Step one, grab a task
            scheduler.wait_for_notif().await;
            let task = scheduler.pop_task().await;

            // Step two, consume task
            println!("{:?}", task);
        });

        self.active_turtles.insert(id, handle);
    }

    pub fn stop_turtle(&mut self, id: u64){
        // Stops a turtle from executing
        if let Some(handle) = self.active_turtles.get(&id) {
            handle.abort();
        }
    }

    pub fn is_turtle_active(&self, id: u64) -> bool {
        self.active_turtles.contains_key(&id)
    }
}