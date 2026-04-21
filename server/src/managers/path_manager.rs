use std::{cmp::Ordering, collections::{BinaryHeap, HashMap, HashSet}};

use crate::turtle::traits::SmartTurtle;
use crate::{managers::{block_manager::BlockManager, turtle_manager::TurtleManager}, util::vector::Vector3};

/// A* stuff
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: Vector3,
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

/// Time cooperation
struct TimeSlice {
    nodes: HashMap<Vector3, u64>,
    edges: HashSet<(Vector3, Vector3)>,
}

impl TimeSlice {
    fn new() -> TimeSlice {
        TimeSlice {
            nodes: HashMap::new(),
            edges: HashSet::new(),
        }
    }

    fn is_occupied(&self, pos: Vector3) -> bool {
        self.nodes.contains_key(&pos)
    }

    fn has_edge(&self, from: Vector3, to: Vector3) -> bool {
        self.edges.contains(&(from, to))
    }
}

/// This handles all the paths within the ledger
pub struct PathManager {
    turtle_manager: TurtleManager,
    block_manager: BlockManager,
    goals: HashMap<u64, Vector3>,
    reservations: Vec<TimeSlice>,
    window: u64,
    tick: usize,
}

impl PathManager {
    pub fn new(block_manager: BlockManager, turtle_manager: TurtleManager) -> PathManager {
        Self {
            turtle_manager,
            block_manager,
            goals: HashMap::new(),
            reservations: Vec::new(),
            window: 32,
            tick: 0,
        }
    }

    fn get_slice(&mut self, t: usize) -> &mut TimeSlice {
        &mut self.reservations[t - self.tick]
    }

    fn advance_tick(&mut self) {
        self.tick += 1;
        self.reservations.remove(0);
        self.reservations.push(TimeSlice::new());
    }

    fn reserve(&mut self, turtle_id: u64, path: Vec<(Vector3, usize)>) -> bool {
        // Try to reserve the given path for the turtle, return true if successful, false if any conflicts
        for i in 0..(path.len() - 1) {
            let (from, t) = path[i];
            let (to, next_t) = path[i + 1];

            // Check node reservation
            if self.get_slice(t).is_occupied(from) || self.get_slice(next_t).is_occupied(to) {
                return false;
            }

            // Check edge reservation (swap check)
            if self.get_slice(t).has_edge(to, from) {
                return false;
            }
        }

        // If we got here, the path is clear. Reserve it.
        for i in 0..(path.len() - 1) {
            let (from, t) = path[i];
            let (to, next_t) = path[i + 1];

            self.get_slice(t).nodes.insert(from, turtle_id);
            self.get_slice(next_t).nodes.insert(to, turtle_id);
            self.get_slice(t).edges.insert((from, to));
        }

        true
    }

    fn dynamic_path(&self, from: Vector3, to: Vector3) -> Option<Vec<(Vector3, usize)>> {
        // WHCA* Implementation
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(Vector3, usize), (Vector3, usize)> = HashMap::new();
        let mut g_score: HashMap<(Vector3, usize), i64> = HashMap::new();

        let t = self.tick; // Relative time estimated for this window request
        
        g_score.insert((from, t), 0);
        open_set.push(Node {
            pos: from,
            f_score: Vector3::manhattan_distance(&from, &to),
        });

        let mut best_node = (from, t);
        let mut best_f = i64::MAX;

        while let Some(Node { pos: current, .. }) = open_set.pop() {
            if current == to {
                best_node = (current, t);
                break;
            }

            // Keep track of the node that gets us closest to the goal if we can't reach it within window
            let h = Vector3::manhattan_distance(&current, &to);
            if h < best_f {
                best_f = h;
                best_node = (current, t);
            }

            if t >= (self.tick + self.window as usize) {
                // We've reached the end of our planning window, stop searching further
                continue;
            }

            // Neighbors: 6 directions + wait
            let mut neighbors = vec![
                Vector3::new(current.x + 1, current.y, current.z),
                Vector3::new(current.x - 1, current.y, current.z),
                Vector3::new(current.x, current.y + 1, current.z),
                Vector3::new(current.x, current.y - 1, current.z),
                Vector3::new(current.x, current.y, current.z + 1),
                Vector3::new(current.x, current.y, current.z - 1),
            ];
            neighbors.push(current); // Wait move

            for neighbor in neighbors {
                let next_t = t + 1;

                // Static collision check (blocks)
                if self.block_manager.get_block(neighbor.x, neighbor.y, neighbor.z).is_some() {
                    continue;
                }

                // Dynamic collision check (ledger)
                if self.reservations[next_t].nodes.contains_key(&neighbor) {
                    continue;
                }

                if self.reservations[t].edges.contains(&(neighbor, current)) {
                    continue;
                }

                // Check g-scores
                let current_g = g_score.get(&(current, t)).cloned().unwrap_or(i64::MAX);
                if current_g == i64::MAX { continue; } // Should not happen

                let tentative_g_score = current_g + 1;
                if tentative_g_score < *g_score.get(&(neighbor, next_t)).unwrap_or(&i64::MAX) {
                    came_from.insert((neighbor, next_t), (current, t));
                    g_score.insert((neighbor, next_t), tentative_g_score);
                    let f_score = tentative_g_score + Vector3::manhattan_distance(&neighbor, &to);
                    open_set.push(Node { pos: neighbor, f_score });
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
        path.push((from, self.tick)); // Add the start node
        path.reverse();

        if path.len() <= 1 && from != to {
            return None;
        }

        Some(path)
    }

    fn static_path(&self, pos: Vector3) -> Vec<(Vector3, usize)> {
        // Reserve the current spot up to window, used for stationary turtles that still need to be considered in pathfinding
        let mut dummy_path = Vec::new();

        for i in self.tick..(self.tick + self.window as usize) {
            dummy_path.push((pos, i));
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

    pub async fn execute(&mut self) -> HashMap<u64, Result<(), String>> {
        // This should be called after every turtle has reserved its path for the current window
        // it will path every turtle to their goal given within this plan
        let mut results = HashMap::new();

        while results.len() < self.goals.len() {
            // Clear ledger for every iteration to ensure fresh reservations for every step
            self.advance_tick();
            self.reservations.clear();

            for _ in 0..self.window {
                self.reservations.push(TimeSlice::new());
            }

            // First, pre-reserve all turtles' current positions at tick to handle initial swap checks
            let all_turtle_ids = self.turtle_manager.iter_ids().await;

            for turtle_id in &all_turtle_ids {
                if let Some(turtle) = self.turtle_manager.get_turtle(*turtle_id).await {
                    let turtle = turtle.lock().await;
                    let current_pos = Vector3::from(turtle.get_position());
                    self.reserve(*turtle_id, vec![(current_pos, self.tick)]);
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
            let ids_and_goals = self.goals.iter().map(|(id, goal)| (*id, *goal)).collect::<Vec<_>>();

            for (turtle_id, goal) in ids_and_goals {
                // Skip if turtle already has a result
                if results.contains_key(&turtle_id) {
                    continue;
                }

                // Get the path reserved
                if let Some(turtle) = self.turtle_manager.get_turtle(turtle_id).await {
                    let turtle = turtle.lock().await;
                    let current_pos = Vector3::from(turtle.get_position());

                    if let Some(path) = self.dynamic_path(current_pos, goal) {
                        self.reserve(turtle_id, path);
                    } else {
                        // The reservation couldnt be made, therefore the path probably doesn't exist.
                        results.insert(turtle_id, Err("No path.".to_string()));
                    }
                } else {
                    // Turtle disappeared, maybe it got destroyed or something, just ignore it for this window
                    results.insert(turtle_id, Err("Couldn't acquire turtle.".to_string()));
                }
            }

            // Loop for every reservation and execute the first step
            let ids_and_next_positions = self.get_slice(self.tick + 1).nodes.iter().map(|(pos, id)| (*id, *pos)).collect::<Vec<_>>();

            for (turtle_id, next) in ids_and_next_positions {
                // Check if this is the goal
                if let Some(goal) = self.goals.get(&turtle_id) {
                    if next == *goal {
                        results.insert(turtle_id, Ok(()));
                        continue;
                    }
                }

                // Acquire turtle control
                if let Some(turtle) = self.turtle_manager.get_turtle(turtle_id).await {
                    let mut turtle = turtle.lock().await;
                    let delta = Vector3::from(next) - Vector3::from(turtle.get_position());
                    
                    match turtle.move_to(delta.x, delta.y, delta.z).await {
                        Ok(()) => {}, // Success, do nothing and wait for next loop to move the next block
                        Err(_) => {
                            // Error occured, but this isnt the end. Scan all the blocks and wait for next iteration
                            let _ = turtle.scan_blocks().await;
                        }
                    }

                    // ! Pause a bit just to debug
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                } else {
                    // Turtle disappeared, maybe it got destroyed or something, just ignore it for this window
                    results.insert(turtle_id, Err("Couldn't acquire turtle.".to_string()));
                }
            }
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
        let path = pl.dynamic_path(from, to).expect("Should find path");
        
        assert!(!path.is_empty());
        assert_eq!(path[0].1, 0);
        assert_eq!(path[0].0.x, 0);
        assert_eq!(path.last().unwrap().0.x, 2);
        assert_eq!(path.last().unwrap().0.y, 0);
        assert_eq!(path.last().unwrap().0.z, 0);
        
        // Check that path is sequential in time starting from 0
        for i in 0..path.len() {
            assert_eq!(path[i].1, i);
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
        let path = pl.dynamic_path(from, to).expect("Should find path");
        
        // Should not contain (1, 0, 0)
        for coord in &path {
            assert!(coord.0.x != 1 || coord.0.y != 0 || coord.0.z != 0);
        }
        
        assert_eq!(path.last().unwrap().0.x, 2);
    }

    #[tokio::test]
    async fn test_dynamic_collision_avoidance() {
        let (_bm, mut pl) = setup("test_dynamic_collision_avoidance").await;
        
        pl.set_window(10);

        // Turtle 1 reserves a path that goes through (1, 0, 0) at t=1
        let t1_from = Vector3::new(0, 0, 0);
        let t1_to = Vector3::new(2, 0, 0);
        let reserved1 = pl.dynamic_path(1, t1_from, t1_to).expect("T1 should find path");
        
        // Turtle 2 tries to reserve a path that would normally go through (1, 0, 0) at t=1
        // (e.g., from (1, 1, 0) to (1, -1, 0))
        let t2_from = Vector3::new(1, 1, 0);
        let t2_to = Vector3::new(1, -1, 0);
        let reserved2 = pl.dynamic_path(2, t2_from, t2_to).expect("T2 should find path");
        
        let path1 = reserved1.get_path();
        let path2 = reserved2.get_path();
        
        // Verify no collisions in space-time
        for c1 in path1 {
            for c2 in path2 {
                if c1.t == c2.t {
                    assert!(c1.x != c2.x || c1.y != c2.y || c1.z != c2.z, "Collision at t={}", c1.t);
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
        let _reserved1 = pl.dynamic_path(1, t1_from, t1_to).expect("T1 path");
        
        // Turtle 2 tries to go (1,0,0) -> (0,0,0) at the same time
        let t2_from = Vector3::new(1, 0, 0);
        let t2_to = Vector3::new(0, 0, 0);
        let reserved2 = pl.dynamic_path(2, t2_from, t2_to).expect("T2 path");
        
        // T2 should HAVE to wait or go around, but not swap directly
        // If it goes around, path length will be > 1
        // If it waits, it might take longer.
        let path2 = reserved2.get_path();
        
        // At t=1, T1 is at (1,0,0). So T2 cannot be at (0,0,0) at t=1 if it means swapping
        // Let's verify T2 path at t=1 isn't (0,0,0)
        let t1_coord = path2.iter().find(|c| c.t == 1).expect("T2 should have a position at t=1");
        assert!(t1_coord.x != 0 || t1_coord.y != 0 || t1_coord.z != 0, "Swap collision detected at t=1");
    }

    #[tokio::test]
    async fn test_reservation_cleanup() {
        let (_bm, mut pl) = setup("test_reservation_cleanup").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(1, 0, 0);
        
        pl.set_window(10);

        let coord_at_t0 = Coord { x: 0, y: 0, z: 0, t: 0 };
        let coord_at_t1 = Coord { x: 1, y: 0, z: 0, t: 1 };
        
        {
            let _reserved = pl.dynamic_path(1, from, to).expect("Path");
            assert!(pl.ledger.contains_key(&coord_at_t0));
            assert!(pl.ledger.contains_key(&coord_at_t1));
        }
        
        // After drop, ledger should be empty
        assert!(!pl.ledger.contains_key(&coord_at_t0));
        assert!(!pl.ledger.contains_key(&coord_at_t1));
    }

    #[tokio::test]
    async fn test_window_constraint() {
        let (_bm, mut pl) = setup("test_window_constraint").await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(10, 0, 0);
        let window = 3;
        
        pl.set_window(window);
        let reserved = pl.dynamic_path(1, from, to).expect("Path");
        let path = reserved.get_path();
        
        // Path length is window + 1 (including t=0)
        assert!(path.len() <= (window + 1) as usize);
        assert!(path.last().unwrap().t <= window as u64);
        
        // Should not have reached the goal (10, 0, 0)
        assert!(path.last().unwrap().x < 10);
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
        let reserved1 = pl.dynamic_path(1, from1, to1).expect("Path 1 should be found");
        let path1 = reserved1.get_path();
        
        // Turtle 2: (0,-2,0) -> (0,2,0) (perpendicular path)
        let from2 = Vector3::new(0, -2, 0);
        let to2 = Vector3::new(0, 2, 0);
        let reserved2 = pl.dynamic_path(2, from2, to2).expect("Path 2 should be found");
        let path2 = reserved2.get_path();
        
        // Verify both paths were found
        assert!(!path1.is_empty());
        assert!(!path2.is_empty());
        
        // Check that paths don't collide at the same time
        for coord1 in path1 {
            for coord2 in path2 {
                if coord1.t == coord2.t {
                    // At the same time step, turtles should not occupy the same position
                    assert!(coord1.x != coord2.x || coord1.y != coord2.y || coord1.z != coord2.z,
                        "Paths collide at position ({}, {}, {}) at time {}", 
                        coord1.x, coord1.y, coord1.z, coord1.t);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_coord_vector_conversions() {
        let v = Vector3::new(1, 2, 3);
        let t = 10;
        let c = Coord::from((v, t));
        
        assert_eq!(c.x, 1);
        assert_eq!(c.y, 2);
        assert_eq!(c.z, 3);
        assert_eq!(c.t, 10);
        
        let v2: Vector3 = c.into();
        assert_eq!(v2.x, 1);
        assert_eq!(v2.y, 2);
        assert_eq!(v2.z, 3);
    }

    #[tokio::test]
    async fn test_stationary_turtle_reservation() {
        let (_bm, pl) = setup("test_stationary_turtle_reservation").await;
        let pos = Vector3::new(5, 5, 5);
        let turtle_id = 42;
        
        {
            let _reserved = pl.static_path(turtle_id, pos);
            
            // Check if all coords in window are reserved
            for t in 0..pl.window {
                let coord = Coord { x: 5, y: 5, z: 5, t };
                assert_eq!(*pl.ledger.get(&coord).unwrap(), turtle_id);
            }
        }
        
        // Ledger should be empty after ReservedPath is dropped
        assert!(pl.ledger.is_empty());
    }

    #[tokio::test]
    async fn test_stationary_turtle_blocks_path() {
        let (_bm, pl) = setup("test_stationary_turtle_blocks_path").await;
        
        let stationary_pos = Vector3::new(1, 0, 0);
        let _stationary_res = pl.static_path(2, stationary_pos);
        
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        // T1 should avoid (1,0,0) because T2 is stationary there
        let reserved1 = pl.dynamic_path(1, from, to).expect("Should find path");
        let path1 = reserved1.get_path();
        
        for coord in path1 {
            assert!(coord.x != 1 || coord.y != 0 || coord.z != 0, "T1 hit stationary T2 at t={}", coord.t);
        }
    }

    #[tokio::test]
    async fn test_set_goal() {
        let (_bm, pl) = setup("test_set_goal").await;
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
        let (_bm, pl) = setup("test_execute_no_turtles").await;
        
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

    #[test]
    fn test_node_ordering() {
        let n1 = Node { pos: Vector3::new(0,0,0), t: 0, f_score: 10 };
        let n2 = Node { pos: Vector3::new(0,0,0), t: 1, f_score: 5 };
        let n3 = Node { pos: Vector3::new(0,0,0), t: 2, f_score: 10 };
        
        let mut heap = BinaryHeap::new();
        heap.push(n1);
        heap.push(n2);
        heap.push(n3);
        
        // Lower f_score should come first
        assert_eq!(heap.pop().unwrap().f_score, 5);
        
        let first = heap.pop().unwrap();
        assert_eq!(first.f_score, 10);
        assert_eq!(first.t, 2);
        
        let second = heap.pop().unwrap();
        assert_eq!(second.f_score, 10);
        assert_eq!(second.t, 0);
    }

    #[tokio::test]
    async fn test_intersection_collision() {
        let (_bm, mut pl) = setup("test_intersection_collision").await;
        pl.set_window(10);

        // Scenario: T1 is at (1,0,0) and has already finished its path (it's "stationary" now)
        // T2 wants to pass through (1,0,0) to get to (2,0,0).
        
        let t1_id = 1;
        let t1_pos = Vector3::new(1, 0, 0);
        
        let t2_id = 2;
        let t2_from = Vector3::new(0, 0, 0);
        let t2_to = Vector3::new(2, 0, 0);

        // In the real execute loop, T1 would be in 'results' or not in 'goals', 
        // and thus reserved as stationary.
        let _res1 = pl.static_path(t1_id, t1_pos);
        
        // T2 plans its path
        let res2 = pl.dynamic_path(t2_id, t2_from, t2_to).expect("T2 should find a path");
        let path2 = res2.get_path();
        
        // T2 should NOT go through (1,0,0) at any time because T1 is there.
        for coord in path2 {
            assert!(coord.x != 1 || coord.y != 0 || coord.z != 0, "T2 collided with stationary T1 at t={}", coord.t);
        }
    }

    #[tokio::test]
    async fn test_dynamic_swap_collision() {
        let (_bm, mut pl) = setup("test_dynamic_swap_collision").await;
        pl.set_window(10);

        // T1 at (0,0,0), T2 at (1,0,0)
        // They want to swap.
        
        // Simulating the execute loop's pre-reservation:
        pl.ledger.insert(Coord { x: 0, y: 0, z: 0, t: 0 }, 1);
        pl.ledger.insert(Coord { x: 1, y: 0, z: 0, t: 0 }, 2);
        
        // T1 plans first (0,0,0 -> 1,0,0)
        let res1 = pl.dynamic_path(1, Vector3::new(0,0,0), Vector3::new(1,0,0)).expect("T1 path");
        let path1 = res1.get_path();
        
        // T2 plans second (1,0,0 -> 0,0,0)
        let res2 = pl.dynamic_path(2, Vector3::new(1,0,0), Vector3::new(0,0,0)).expect("T2 path");
        let path2 = res2.get_path();
        
        // Check for swap at t=1
        // T1: (0,0,0, t=0) -> (1,0,0, t=1)
        // T2: (1,0,0, t=0) -> (0,0,0, t=1)  <-- This should be blocked by swap check
        
        let t1_pos_at_1 = path1.iter().find(|c| c.t == 1).unwrap();
        let t2_pos_at_1 = path2.iter().find(|c| c.t == 1).unwrap();
        
        // One of them must have waited or moved elsewhere
        assert!(!(t1_pos_at_1.x == 1 && t2_pos_at_1.x == 0), "Swap collision detected at t=1");
    }
}
