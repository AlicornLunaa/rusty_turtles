use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};

use rustc_hash::FxHashMap;

use crate::turtle::traits::SmartTurtle;
use crate::{managers::{block_manager::BlockManager, turtle_manager::TurtleManager}, util::vector::Vector3};

// Space-time coordinate
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coord {
    pub pos: Vector3,
    pub tick: usize,
}

type ReservationMap = FxHashMap<Coord, u64>;
type TransitionMap = FxHashMap<(Vector3, Vector3, usize), u64>;

/// A* stuff
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    coord: Coord,
    f_score: i64,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// This handles all the paths within the ledger
pub struct PathManager {
    turtle_manager: TurtleManager,
    block_manager: BlockManager,
    goals: FxHashMap<u64, Vector3>,
    reservations: ReservationMap,
    transitions: TransitionMap,
    window: u64,
    tick: usize,
}

impl PathManager {
    pub fn new(block_manager: BlockManager, turtle_manager: TurtleManager) -> PathManager {
        Self {
            turtle_manager,
            block_manager,
            goals: FxHashMap::default(),
            reservations: FxHashMap::default(),
            transitions: FxHashMap::default(),
            window: 32,
            tick: 0,
        }
    }

    fn reserve(&mut self, turtle_id: u64, path: Vec<Coord>) -> bool {
        // Try to reserve the given path for the turtle, return true if successful, false if any conflicts
        // First check for conflicts
        for coord in &path {
            if let Some(other_turtle) = self.reservations.get(coord) {
                if *other_turtle != turtle_id {
                    return false; // Already reserved by another turtle
                }
            }
        }

        // Then check for swap collisions
        for i in 0..(path.len() - 1) {
            let from = path[i];
            let to = path[i + 1];

            if let Some(other_turtle) = self.transitions.get(&(from.pos, to.pos, from.tick)) {
                if *other_turtle != turtle_id {
                    return false; // Swap collision
                }
            }

            if let Some(other_turtle) = self.transitions.get(&(to.pos, from.pos, from.tick)) {
                if *other_turtle != turtle_id {
                    return false; // Swap collision
                }
            }
        }

        // Save everything
        for coord in &path {
            self.reservations.insert(*coord, turtle_id);
        }

        for i in 0..(path.len() - 1) {
            let from = path[i];
            let to = path[i + 1];
            self.transitions.insert((from.pos, to.pos, from.tick), turtle_id);
        }

        true
    }

    fn dynamic_path(&self, turtle_id: u64, from: Vector3, to: Vector3) -> Option<Vec<Coord>> {
        // A* solving for a path from 'from' to 'to' while avoiding static blocks and dynamic reservations in the ledger
        let mut open_set = BinaryHeap::new();
        let mut came_from: FxHashMap<Coord, Coord> = FxHashMap::default();
        let mut g_score: FxHashMap<Coord, i64> = FxHashMap::default();

        let now = chrono::Utc::now().timestamp();
        let start_tick = self.tick;
        let start_coord = Coord { pos: from, tick: start_tick };
        
        g_score.insert(start_coord, 0);
        open_set.push(Node {
            coord: start_coord,
            f_score: Vector3::manhattan_distance(&from, &to),
        });

        let mut best_node = start_coord;
        let mut best_f = i64::MAX;

        while let Some(Node { coord: current, f_score: _ }) = open_set.pop() {
            // If we reached the goal, reconstruct path immediately
            if current.pos == to {
                best_node = Coord { pos: to, tick: current.tick };
                break;
            }

            // Keep track of the node that gets us closest to the goal if we can't reach it within window
            let h = Vector3::manhattan_distance(&current.pos, &to);
            if h < best_f {
                best_f = h;
                best_node = Coord { pos: current.pos, tick: current.tick };
            }

            // We've reached the end of our planning window, stop searching further
            if current.tick >= (self.tick + self.window as usize) {
                break;
            }

            // Neighbors: 6 directions + wait
            let neighbors = vec![
                Vector3::new(current.pos.x + 1, current.pos.y, current.pos.z),
                Vector3::new(current.pos.x - 1, current.pos.y, current.pos.z),
                Vector3::new(current.pos.x, current.pos.y + 1, current.pos.z),
                Vector3::new(current.pos.x, current.pos.y - 1, current.pos.z),
                Vector3::new(current.pos.x, current.pos.y, current.pos.z + 1),
                Vector3::new(current.pos.x, current.pos.y, current.pos.z - 1),
                current.pos // Wait in place option
            ];

            for neighbor in neighbors {
                let next_tick = current.tick + 1;
                let next_coord = Coord { pos: neighbor, tick: next_tick };
                let mut turtle_in_way = false;
                let mut unknown_path = true;
                let mut last_updated = 0;

                // Static collision check (blocks)
                if let Some(block) = self.block_manager.get_block(neighbor.x, neighbor.y, neighbor.z) {
                    // Block must not be air and have been updated within the last 5 minutes
                    let five_minutes_ago = now - 300;

                    if block.block_type == "computercraft:turtle" || block.block_type == "computercraft:turtle_advanced" {
                        turtle_in_way = true;
                    }

                    if block.block_type == "minecraft:air" {
                        // We know this path is clear, it should be prioritized a little
                        unknown_path = false;
                    }

                    // If the block was updated more than 5 minutes ago, consider it unknown to allow for natural terrain changes, but still prefer known paths
                    last_updated = (now - block.last_updated) / 60; // The older the block, the less we trust it, up to a maximum of 5 minutes where we consider it completely unknown

                    if !turtle_in_way && block.block_type != "minecraft:air" && block.last_updated >= five_minutes_ago {
                        continue;
                    }
                }

                // Dynamic collision check (ledger)
                if let Some(other_turtle) = self.reservations.get(&next_coord) {
                    if *other_turtle != turtle_id {
                        // Another turtle is already reserved here at this time
                        continue;
                    }
                }

                if let Some(other_turtle) = self.transitions.get(&(current.pos, neighbor, current.tick)) {
                    if *other_turtle != turtle_id {
                        // Another turtle is moving from current.pos to neighbor at the same time
                        continue;
                    }
                }

                if let Some(other_turtle) = self.transitions.get(&(neighbor, current.pos, current.tick)) {
                    if *other_turtle != turtle_id {
                        // Another turtle is moving from neighbor to current.pos at the same time
                        continue;
                    }
                }

                // Check g-scores
                let current_g = g_score.get(&current).cloned().unwrap_or(i64::MAX);
                if current_g == i64::MAX { continue; } // Should not happen

                // Determine cost for waiting vs moving and known vs unknown blocks
                let mut tentative_g_score = current_g;

                if neighbor != current.pos {
                    tentative_g_score += 2; // Moving costs 2
                } else {
                    tentative_g_score += 1; // Waiting costs 1
                }

                if unknown_path {
                    tentative_g_score += 2; // Heavily penalize paths through unknown blocks to encourage known paths, but still allow if no other options
                }

                tentative_g_score -= last_updated / 2; // The older the block information, the less we trust it, so we reduce the cost the older it is to encourage using it but still prefer newer information
                tentative_g_score = tentative_g_score.max(current_g + 1); // Ensure that we always prefer moving forward in time, even if the heuristic is bad, to prevent infinite loops

                // Check scores
                if tentative_g_score < *g_score.get(&next_coord).unwrap_or(&i64::MAX) {
                    came_from.insert(next_coord, current);
                    g_score.insert(next_coord, tentative_g_score);
                    let f_score = tentative_g_score + Vector3::manhattan_distance(&neighbor, &to);
                    open_set.push(Node { coord: next_coord, f_score });
                }
            }
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut curr = best_node;
        while let Some(&prev) = came_from.get(&curr) {
            path.push(curr);
            curr = prev;
        }
        path.push(Coord { pos: from, tick: start_tick }); // Add the start node
        path.reverse();

        if path.len() <= 1 && from != to {
            return None;
        }

        Some(path)
    }

    fn static_path(&self, pos: Vector3) -> Vec<Coord> {
        // Reserve the current spot up to window, used for stationary turtles that still need to be considered in pathfinding
        let mut dummy_path = Vec::new();

        for i in self.tick..=(self.tick + self.window as usize) {
            dummy_path.push(Coord { pos, tick: i });
        }

        dummy_path
    }

    pub fn set_window(&mut self, window: u64) {
        self.window = window;
    }

    pub fn set_goal(&mut self, turtle_id: u64, to: Vector3) {
        // Sets this turtle's goal before execution of a path
        self.goals.insert(turtle_id, to);
    }

    pub async fn execute_step(&mut self, results: &mut HashMap<u64, Result<(), String>>, paths: &mut HashMap<u64, Vec<Coord>>) {
        // Reset state for this step of planning
        self.reservations.clear();
        self.transitions.clear();
        paths.clear();
        
        // Check for existence and goal completion
        let all_turtle_ids = self.turtle_manager.iter_ids().await;
        let ids_in_goals: Vec<u64> = self.goals.keys().cloned().collect();

        for turtle_id in ids_in_goals {
            if results.contains_key(&turtle_id) { continue; }

            if let Some(turtle) = self.turtle_manager.get_turtle(turtle_id).await {
                let turtle = turtle.lock().await;
                let current_pos = Vector3::from(turtle.get_position());
                if let Some(goal) = self.goals.get(&turtle_id) {
                    if current_pos == *goal {
                        results.insert(turtle_id, Ok(()));
                    }
                }
            } else {
                results.insert(turtle_id, Err("Couldn't acquire turtle.".to_string()));
            }
        }

        if results.len() >= self.goals.len() {
            return;
        }

        // Pre-reserve all turtles' current positions
        for turtle_id in &all_turtle_ids {
            if let Some(turtle) = self.turtle_manager.get_turtle(*turtle_id).await {
                let turtle = turtle.lock().await;
                let current_pos = Vector3::from(turtle.get_position());
                let path = vec![Coord { pos: current_pos, tick: self.tick }, Coord { pos: current_pos, tick: self.tick + 1 }];
                self.reserve(*turtle_id, path);
            }
        }

        // Identify and reserve stationary turtles (including those that reached goal or failed)
        for turtle_id in &all_turtle_ids {
            let is_dynamic = self.goals.contains_key(turtle_id) && !results.contains_key(turtle_id);

            if !is_dynamic {
                if let Some(turtle) = self.turtle_manager.get_turtle(*turtle_id).await {
                    let turtle = turtle.lock().await;
                    let current_pos = Vector3::from(turtle.get_position());
                    self.reserve(*turtle_id, self.static_path(current_pos));
                }
            }
        }
        
        // For every turtle in this plan, get a reservation for its path to the goal
        let ids_and_goals = self.goals.iter()
            .filter(|(id, _)| !results.contains_key(id))
            .map(|(id, goal)| (*id, *goal))
            .collect::<Vec<_>>();

        for (turtle_id, goal) in ids_and_goals {
            if let Some(turtle) = self.turtle_manager.get_turtle(turtle_id).await {
                let turtle = turtle.lock().await;
                let current_pos = Vector3::from(turtle.get_position());

                if let Some(path) = self.dynamic_path(turtle_id, current_pos, goal) {
                    if self.reserve(turtle_id, path.clone()) {
                        paths.insert(turtle_id, path);
                    }
                } else {
                    // The reservation couldnt be made.
                    results.insert(turtle_id, Err("No path.".to_string()));
                }
            }
        }

        // Loop for every reservation and execute the first step
        let mut handles = Vec::new();
        
        for (turtle_id, path) in paths {
            // Acquire turtle control
            let turtle = self.turtle_manager.get_turtle(*turtle_id).await;

            let action = async move {
                if let Some(turtle) = turtle {
                    let mut turtle = turtle.lock().await;
                    let current_pos = Vector3::from(turtle.get_position());
                    let delta = path[1].pos - current_pos;
                    
                    if delta.x != 0 || delta.y != 0 || delta.z != 0 {
                        match turtle.move_to(delta.x, delta.y, delta.z).await {
                            Ok(()) => {
                                // Success, lets just gather information just because
                                let _ = turtle.scan_blocks().await;
                            }, 
                            Err(_) => {
                                // Error occured, scan blocks and try again next iteration
                                let _ = turtle.scan_blocks().await;
                            }
                        }
                    }
                }
            };
            
            handles.push(action);
        }
        
        futures_util::future::join_all(handles).await;
    }

    pub async fn execute(&mut self) -> HashMap<u64, Result<(), String>> {
        // This should be called after every turtle has reserved its path for the current window
        // it will path every turtle to their goal given within this plan
        let mut results = HashMap::new();
        let mut paths = HashMap::new();

        while results.len() < self.goals.len() {
            // Reset state for this step of planning
            self.execute_step(&mut results, &mut paths).await;
        }
        
        // Execution is done, every turtle should maybe be at the goal
        self.goals.clear();

        // Return the result of all the turtle's pathing
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn setup(db_name: &str) -> (BlockManager, PathManager) {
        unsafe {
            env::set_var("DATABASE_URL", format!("file:{}?mode=memory&cache=shared", db_name));
        }
        let bm = BlockManager::new().await;
        let tm = TurtleManager::new();
        let pl = PathManager::new(bm.clone(), tm.clone());
        (bm, pl)
    }

    #[tokio::test]
    async fn test_simple_path() {
        let (_bm, mut pl) = setup("test_simple_path").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        pl.set_window(10);
        let path = pl.dynamic_path(1, from, to).expect("Should find path");
        
        assert!(!path.is_empty());
        assert_eq!(path[0].tick, 0);
        assert_eq!(path[0].pos.x, 0);
        assert_eq!(path.last().unwrap().pos.x, 2);
        assert_eq!(path.last().unwrap().pos.y, 0);
        assert_eq!(path.last().unwrap().pos.z, 0);
        
        // Check that path is sequential in time starting from 0
        for i in 0..path.len() {
            assert_eq!(path[i].tick, i);
        }
    }

    #[tokio::test]
    async fn test_obstacle_avoidance() {
        let (bm, mut pl) = setup("test_obstacle_avoidance").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        // Place an obstacle at (1, 0, 0)
        bm.update_block(1, 0, 0, "stone".to_string()).await;
        
        pl.set_window(10);
        let path = pl.dynamic_path(1, from, to).expect("Should find path");
        
        // Should not contain (1, 0, 0)
        for coord in &path {
            assert!(coord.pos.x != 1 || coord.pos.y != 0 || coord.pos.z != 0);
        }
        
        assert_eq!(path.last().unwrap().pos.x, 2);
    }

    #[tokio::test]
    async fn test_dynamic_collision_avoidance() {
        let (_bm, mut pl) = setup("test_dynamic_collision_avoidance").await;
        
        pl.set_window(10);

        // Turtle 1 reserves a path that goes through (1, 0, 0) at t=1
        let t1_from = Vector3::new(0, 0, 0);
        let t1_to = Vector3::new(2, 0, 0);
        let path1 = pl.dynamic_path(1, t1_from, t1_to).expect("T1 should find path");
        pl.reserve(1, path1.clone());
        
        // Turtle 2 tries to reserve a path that would normally go through (1, 0, 0) at t=1
        // (e.g., from (1, 1, 0) to (1, -1, 0))
        let t2_from = Vector3::new(1, 1, 0);
        let t2_to = Vector3::new(1, -1, 0);
        let path2 = pl.dynamic_path(2, t2_from, t2_to).expect("T2 should find path");
        
        // Verify no collisions in space-time
        for c1 in &path1 {
            for c2 in &path2 {
                if c1.tick == c2.tick {
                    assert!(c1.pos.x != c2.pos.x || c1.pos.y != c2.pos.y || c1.pos.z != c2.pos.z, "Collision at t={}", c1.tick);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_swap_collision_avoidance() {
        let (_bm, mut pl) = setup("test_swap_collision_avoidance").await;
        
        pl.set_window(10);

        // Turtle 1 goes (0,0,0) -> (1,0,0)
        let t1_from = Vector3::new(0, 0, 0);
        let t1_to = Vector3::new(1, 0, 0);
        let path1 = pl.dynamic_path(1, t1_from, t1_to).expect("T1 path");
        pl.reserve(1, path1);
        
        // Turtle 2 tries to go (1,0,0) -> (0,0,0) at the same time
        let t2_from = Vector3::new(1, 0, 0);
        let t2_to = Vector3::new(0, 0, 0);
        let path2 = pl.dynamic_path(2, t2_from, t2_to).expect("T2 path");
        
        // T2 should HAVE to wait or go around, but not swap directly
        // At t=1, T1 is at (1,0,0). So T2 cannot be at (0,0,0) at t=1 if it means swapping
        // Let's verify T2 path at t=1 isn't (0,0,0)
        let t1_coord = path2.iter().find(|c| c.tick == 1).expect("T2 should have a position at t=1");
        assert!(t1_coord.pos.x != 0 || t1_coord.pos.y != 0 || t1_coord.pos.z != 0, "Swap collision detected at t=1");
    }

    #[tokio::test]
    async fn test_window_constraint() {
        let (_bm, mut pl) = setup("test_window_constraint").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(10, 0, 0);
        let window = 3;
        
        pl.set_window(window);
        let path = pl.dynamic_path(1, from, to).expect("Path");
        
        // Path length is window + 1 (including t=0)
        assert!(path.len() <= (window + 1) as usize);
        assert!(path.last().unwrap().tick <= window as usize);
        
        // Should not have reached the goal (10, 0, 0)
        assert!(path.last().unwrap().pos.x < 10);
    }

    #[tokio::test]
    async fn test_no_path_found() {
        let (bm, mut pl) = setup("test_no_path_found").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        // Surround (0,0,0) with obstacles
        bm.update_block(1, 0, 0, "stone".to_string()).await;
        bm.update_block(-1, 0, 0, "stone".to_string()).await;
        bm.update_block(0, 1, 0, "stone".to_string()).await;
        bm.update_block(0, -1, 0, "stone".to_string()).await;
        bm.update_block(0, 0, 1, "stone".to_string()).await;
        bm.update_block(0, 0, -1, "stone".to_string()).await;
        
        pl.set_window(10);
        let result = pl.dynamic_path(1, from, to);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_two_paths_avoid_each_other() {
        let (_bm, mut pl) = setup("test_two_paths_avoid_each_other").await;
        
        pl.set_window(10);

        // Turtle 1: (-2,0,0) -> (2,0,0)
        let from1 = Vector3::new(-2, 0, 0);
        let to1 = Vector3::new(2, 0, 0);
        let path1 = pl.dynamic_path(1, from1, to1).expect("Path 1 should be found");
        pl.reserve(1, path1.clone());
        
        // Turtle 2: (0,-2,0) -> (0,2,0) (perpendicular path)
        let from2 = Vector3::new(0, -2, 0);
        let to2 = Vector3::new(0, 2, 0);
        let path2 = pl.dynamic_path(2, from2, to2).expect("Path 2 should be found");
        
        // Verify both paths were found
        assert!(!path1.is_empty());
        assert!(!path2.is_empty());
        
        // Check that paths don't collide at the same time
        for coord1 in &path1 {
            for coord2 in &path2 {
                if coord1.tick == coord2.tick {
                    // At the same time step, turtles should not occupy the same position
                    assert!(coord1.pos.x != coord2.pos.x || coord1.pos.y != coord2.pos.y || coord1.pos.z != coord2.pos.z,
                        "Paths collide at position ({}, {}, {}) at time {}", 
                        coord1.pos.x, coord1.pos.y, coord1.pos.z, coord1.tick);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_stationary_turtle_reservation() {
        let (_bm, mut pl) = setup("test_stationary_turtle_reservation").await;
        let pos = Vector3::new(5, 5, 5);
        let turtle_id = 42;
        
        let path = pl.static_path(pos);
        pl.reserve(turtle_id, path);
        
        // Check if all coords in window are reserved
        for t in 0..=pl.window as usize {
            let coord = Coord { pos, tick: pl.tick + t };
            assert!(pl.reservations.contains_key(&coord), "Stationary turtle not reserved at tick {}", coord.tick);
            assert_eq!(pl.reservations.get(&coord).unwrap(), &turtle_id, "Wrong turtle reserved at tick {}", coord.tick);
        }
    }

    #[tokio::test]
    async fn test_stationary_turtle_blocks_path() {
        let (_bm, mut pl) = setup("test_stationary_turtle_blocks_path").await;
        
        let stationary_pos = Vector3::new(1, 0, 0);
        let stat_path = pl.static_path(stationary_pos);
        pl.reserve(2, stat_path);
        
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);

        // T1 should avoid (1,0,0) because T2 is stationary there
        let path1 = pl.dynamic_path(1, from, to).expect("Should find path");
        
        for coord in path1 {
            assert!(coord.pos.x != 1 || coord.pos.y != 0 || coord.pos.z != 0, "T1 hit stationary T2 at t={}", coord.tick);
        }
    }

    #[tokio::test]
    async fn test_set_goal() {
        let (_bm, mut pl) = setup("test_set_goal").await;
        let turtle_id = 1;
        let goal = Vector3::new(10, 20, 30);
        
        pl.set_goal(turtle_id, goal);
        assert_eq!(*pl.goals.get(&turtle_id).unwrap(), goal);
        
        // Clear goals test
        pl.goals.clear();
        assert!(pl.goals.is_empty());
    }

    #[tokio::test]
    async fn test_execute_no_turtles() {
        let (_bm, mut pl) = setup("test_execute_no_turtles").await;
        
        pl.set_goal(1, Vector3::new(1, 0, 0));
        pl.set_goal(2, Vector3::new(0, 1, 0));
        
        // Execute when no turtles are actually in the TurtleManager
        let results = pl.execute().await;
        
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(&1).unwrap().as_ref().unwrap_err(), "Couldn't acquire turtle.");
        assert_eq!(results.get(&2).unwrap().as_ref().unwrap_err(), "Couldn't acquire turtle.");
        
        // Goals should be cleared after execute
        assert!(pl.goals.is_empty());
    }

    #[tokio::test]
    async fn test_intersection_collision() {
        let (_bm, mut pl) = setup("test_intersection_collision").await;
        pl.set_window(10);

        // Scenario: T1 is at (1,0,0) and has already finished its path (it's "stationary" now)
        // T2 wants to pass through (1,0,0) to get to (2,0,0).
        
        let t1_id = 1;
        let t1_pos = Vector3::new(1, 0, 0);
        
        let t2_from = Vector3::new(0, 0, 0);
        let t2_to = Vector3::new(2, 0, 0);

        // In the real execute loop, T1 would be in 'results' or not in 'goals', 
        // and thus reserved as stationary.
        let stat_path = pl.static_path(t1_pos);
        pl.reserve(t1_id, stat_path);
        
        // T2 plans its path
        let path2 = pl.dynamic_path(2, t2_from, t2_to).expect("T2 should find a path");

        
        // T2 should NOT go through (1,0,0) at any time because T1 is there.
        for coord in path2 {
            assert!(coord.pos.x != 1 || coord.pos.y != 0 || coord.pos.z != 0, "T2 collided with stationary T1 at t={}", coord.tick);
        }
    }

    #[tokio::test]
    async fn test_dynamic_swap_collision() {
        let (_bm, mut pl) = setup("test_dynamic_swap_collision").await;
        pl.set_window(10);

        // T1 at (0,0,0), T2 at (1,0,0)
        // They want to swap.
        
        // Simulating the execute loop's pre-reservation:
        let p1 = vec![Coord { pos: Vector3::new(0,0,0), tick: 0 }];
        let p2 = vec![Coord { pos: Vector3::new(1,0,0), tick: 0 }];
        pl.reserve(1, p1);
        pl.reserve(2, p2);
        
        // T1 plans first (0,0,0 -> 1,0,0)
        let path1 = pl.dynamic_path(1, Vector3::new(0,0,0), Vector3::new(1,0,0)).expect("T1 path");
        pl.reserve(1, path1.clone());

        // T2 plans second (1,0,0 -> 0,0,0)
        let path2 = pl.dynamic_path(2, Vector3::new(1,0,0), Vector3::new(0,0,0)).expect("T2 path");

        // Check for swap at t=1
        // T1: (0,0,0, t=0) -> (1,0,0, t=1)
        // T2: (1,0,0, t=0) -> (0,0,0, t=1)  <-- This should be blocked by swap check
        
        let t1_pos_at_1 = path1.iter().find(|c| c.tick == 1).unwrap();
        let t2_pos_at_1 = path2.iter().find(|c| c.tick == 1).unwrap();
        
        // One of them must have waited or moved elsewhere
        assert!(!(t1_pos_at_1.pos.x == 1 && t2_pos_at_1.pos.x == 0), "Swap collision detected at t=1");
    }
}
