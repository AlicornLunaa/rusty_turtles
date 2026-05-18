use std::sync::Arc;

use dashmap::DashMap;
use tokio::task::JoinHandle;

use crate::{managers::{path_manager::AsyncPathManager, turtle_manager::{self, TurtleManager}}, scheduler::{TaskAction, TaskScheduler}, util::vector::Vector3};

#[derive(Clone)]
pub struct SchedulerSystem {
    scheduler: TaskScheduler,
    turtles: TurtleManager,
    paths: AsyncPathManager,
    active_turtles: Arc<DashMap<u64, JoinHandle<()>>>
}

impl SchedulerSystem {
    // Private dispatching for actions
    async fn move_to(paths: &AsyncPathManager, turtle_id: u64, x: i64, y: i64, z: i64) -> bool {
        paths.path(turtle_id, Vector3::new(x, y, z)).await.is_ok()
    }

    async fn dispatch(paths: &AsyncPathManager, turtle_id: u64, task: TaskAction) -> bool {
        match task {
            TaskAction::MoveTo { x, y, z } => Self::move_to(paths, turtle_id, x, y, z).await,
            TaskAction::Craft { items } => todo!(),
            TaskAction::Place { x, y, z, block } => todo!(),
            TaskAction::Break { x, y, z } => todo!(),
            TaskAction::Drop { x, y, z, item } => todo!(),
            TaskAction::Suck { x, y, z } => todo!(),
            TaskAction::Chain { list } => todo!(),
        }
    }

    // Scheduler functions
    pub fn new(scheduler: TaskScheduler, turtle_manager: TurtleManager, paths: AsyncPathManager) -> Self {
        Self {
            scheduler,
            turtles: turtle_manager,
            paths,
            active_turtles: Arc::new(DashMap::new())
        }
    }

    pub fn start_turtle(&mut self, id: u64){
        // Starts the thread for controlling this one turtle
        let handle = tokio::spawn({
            let scheduler = self.scheduler.clone();
            let turtles = self.turtles.clone();
            let paths = self.paths.clone();

            async move {
                // This is the control loop thread, it will start the turtle runtime execution
                loop {
                    // Step one, grab a task
                    if let Some(task) = scheduler.pop_task().await {
                        // Step two, do the task
                        println!("Turtle {} running {:?}", id, task);
    
                        let success = Self::dispatch(&paths, id, task).await;
                        println!("Turtle {} finished task with success flag: {}", id, success);
                    }
                    
                    // Wait for new tasks and repeat
                    if scheduler.len().await == 0 {
                        println!("Turtle {} waiting for new tasks", id);
                        scheduler.wait_for_notif().await;
                    }
                }
            }
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

impl Drop for SchedulerSystem {
    fn drop(&mut self) {
        // Stop all turtles when the system is dropped
        let ids: Vec<u64> = self.active_turtles.iter().map(|entry| *entry.key()).collect();

        for id in ids {
            self.stop_turtle(id);
        }
    }
}