use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}, sync::Arc};

use dashmap::DashMap;

use crate::{managers::block_manager::BlockManager, util::vector::Vector3};

/// 4D coordinate for cooperative A* in 3d space
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Coord { pub x: i64, pub y: i64, pub z: i64, pub t: u64 }

impl From<(Vector3, u64)> for Coord {
    fn from((pos, t): (Vector3, u64)) -> Self {
        Self { x: pos.x, y: pos.y, z: pos.z, t }
    }
}

impl From<Coord> for Vector3 {
    fn from(coord: Coord) -> Self {
        Self::new(coord.x, coord.y, coord.z)
    }
}

/// This holds a reservation within the ledger as long as it exists
pub struct ReservedPath {
    ledger: PathLedger,
    path: Vec<Coord>,
}

impl ReservedPath {
    pub fn get_path(&self) -> &Vec<Coord> {
        &self.path
    }
}

impl Drop for ReservedPath {
    fn drop(&mut self) {
        self.ledger.drop_path_reservation(&self.path);
    }
}

/// A* stuff
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: Vector3,
    t: u64,
    f_score: i64,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
            .then_with(|| self.t.cmp(&other.t))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// This handles all the paths within the ledger
#[derive(Clone)]
pub struct PathLedger {
    ledger: Arc<DashMap<Coord, u64>>, // Reserved blocks and when for turtle id
    block_manager: BlockManager,
}

impl PathLedger {
    pub fn new(block_manager: BlockManager) -> PathLedger {
        Self {
            ledger: Arc::new(DashMap::new()),
            block_manager: block_manager,
        }
    }

    pub fn reserve_path(&self, turtle_id: u64, from: Vector3, to: Vector3, window: u32) -> Option<ReservedPath> {
        // WHCA* Implementation
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(Vector3, u64), (Vector3, u64)> = HashMap::new();
        let mut g_score: HashMap<(Vector3, u64), i64> = HashMap::new();

        let start_t = 0; // Relative time for this window
        
        g_score.insert((from, start_t), 0);
        open_set.push(Node {
            pos: from,
            t: start_t,
            f_score: Vector3::manhattan_distance(&from, &to),
        });

        let mut best_node = (from, start_t);
        let mut best_f = i64::MAX;

        while let Some(Node { pos: current, t, .. }) = open_set.pop() {
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

            if t >= window as u64 {
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
                let coord = Coord::from((neighbor, next_t));

                // Static collision check
                if self.block_manager.get_block(neighbor.x, neighbor.y, neighbor.z).is_some() {
                    continue;
                }

                // Dynamic collision check
                if let Some(res_id) = self.ledger.get(&coord) {
                    if *res_id != turtle_id {
                        continue;
                    }
                }

                // Swap collision check
                let swap_coord = Coord::from((current, next_t));
                if let Some(res_id_at_neighbor_prev) = self.ledger.get(&Coord::from((neighbor, t))) {
                    if *res_id_at_neighbor_prev != turtle_id {
                        if let Some(res_id_at_current_next) = self.ledger.get(&swap_coord) {
                            if *res_id_at_current_next == *res_id_at_neighbor_prev {
                                continue;
                            }
                        }
                    }
                }

                let tentative_g_score = g_score.get(&(current, t)).unwrap() + 1;
                if tentative_g_score < *g_score.get(&(neighbor, next_t)).unwrap_or(&i64::MAX) {
                    came_from.insert((neighbor, next_t), (current, t));
                    g_score.insert((neighbor, next_t), tentative_g_score);
                    let f_score = tentative_g_score + Vector3::manhattan_distance(&neighbor, &to);
                    open_set.push(Node { pos: neighbor, t: next_t, f_score });
                }
            }
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut curr = best_node;
        while let Some(&prev) = came_from.get(&curr) {
            path.push(Coord::from(curr));
            curr = prev;
        }
        path.push(Coord::from((from, start_t)));
        path.reverse();

        if path.len() <= 1 && from != to {
            return None;
        }

        // Reserve in ledger
        for coord in &path {
            self.ledger.insert(*coord, turtle_id);
        }

        Some(ReservedPath {
            ledger: self.clone(),
            path,
        })
    }

    fn drop_path_reservation(&self, coordinates: &Vec<Coord>){
        // Removes all the reservations in the ledger for the coordinate given
        for coord in coordinates {
            let _ = self.ledger.remove(&coord);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn setup() -> (BlockManager, PathLedger) {
        unsafe {
            env::set_var("DATABASE_URL", "file:path_test?mode=memory&cache=shared");
        }
        let bm = BlockManager::new().await;
        let pl = PathLedger::new(bm.clone());
        (bm, pl)
    }

    #[tokio::test]
    async fn test_simple_path() {
        let (_bm, pl) = setup().await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        let reserved = pl.reserve_path(1, from, to, 10).expect("Should find path");
        let path = reserved.get_path();
        
        assert!(!path.is_empty());
        assert_eq!(path[0].t, 0);
        assert_eq!(path[0].x, 0);
        assert_eq!(path.last().unwrap().x, 2);
        assert_eq!(path.last().unwrap().y, 0);
        assert_eq!(path.last().unwrap().z, 0);
        
        // Check that path is sequential in time starting from 0
        for i in 0..path.len() {
            assert_eq!(path[i].t, i as u64);
        }
    }

    #[tokio::test]
    async fn test_obstacle_avoidance() {
        let (bm, pl) = setup().await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        // Place an obstacle at (1, 0, 0)
        bm.update_block(1, 0, 0, "stone".to_string()).await;
        
        let reserved = pl.reserve_path(1, from, to, 10).expect("Should find path");
        let path = reserved.get_path();
        
        // Should not contain (1, 0, 0)
        for coord in path {
            assert!(coord.x != 1 || coord.y != 0 || coord.z != 0);
        }
        
        assert_eq!(path.last().unwrap().x, 2);
    }

    #[tokio::test]
    async fn test_dynamic_collision_avoidance() {
        let (_bm, pl) = setup().await;
        
        // Turtle 1 reserves a path that goes through (1, 0, 0) at t=1
        let t1_from = Vector3::new(0, 0, 0);
        let t1_to = Vector3::new(2, 0, 0);
        let reserved1 = pl.reserve_path(1, t1_from, t1_to, 10).expect("T1 should find path");
        
        // Turtle 2 tries to reserve a path that would normally go through (1, 0, 0) at t=1
        // (e.g., from (1, 1, 0) to (1, -1, 0))
        let t2_from = Vector3::new(1, 1, 0);
        let t2_to = Vector3::new(1, -1, 0);
        let reserved2 = pl.reserve_path(2, t2_from, t2_to, 10).expect("T2 should find path");
        
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
        let (_bm, pl) = setup().await;
        
        // Turtle 1 goes (0,0,0) -> (1,0,0)
        let t1_from = Vector3::new(0, 0, 0);
        let t1_to = Vector3::new(1, 0, 0);
        let _reserved1 = pl.reserve_path(1, t1_from, t1_to, 10).expect("T1 path");
        
        // Turtle 2 tries to go (1,0,0) -> (0,0,0) at the same time
        let t2_from = Vector3::new(1, 0, 0);
        let t2_to = Vector3::new(0, 0, 0);
        let reserved2 = pl.reserve_path(2, t2_from, t2_to, 10).expect("T2 path");
        
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
        let (_bm, pl) = setup().await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(1, 0, 0);
        
        let coord_at_t0 = Coord { x: 0, y: 0, z: 0, t: 0 };
        let coord_at_t1 = Coord { x: 1, y: 0, z: 0, t: 1 };
        
        {
            let _reserved = pl.reserve_path(1, from, to, 10).expect("Path");
            assert!(pl.ledger.contains_key(&coord_at_t0));
            assert!(pl.ledger.contains_key(&coord_at_t1));
        }
        
        // After drop, ledger should be empty
        assert!(!pl.ledger.contains_key(&coord_at_t0));
        assert!(!pl.ledger.contains_key(&coord_at_t1));
    }

    #[tokio::test]
    async fn test_window_constraint() {
        let (_bm, pl) = setup().await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(10, 0, 0);
        let window = 3;
        
        let reserved = pl.reserve_path(1, from, to, window).expect("Path");
        let path = reserved.get_path();
        
        // Path length is window + 1 (including t=0)
        assert!(path.len() <= (window + 1) as usize);
        assert!(path.last().unwrap().t <= window as u64);
        
        // Should not have reached the goal (10, 0, 0)
        assert!(path.last().unwrap().x < 10);
    }

    #[tokio::test]
    async fn test_no_path_found() {
        let (bm, pl) = setup().await;
        let from = Vector3::new(0, 0, 0);
        let to = Vector3::new(2, 0, 0);
        
        // Surround (0,0,0) with obstacles
        bm.update_block(1, 0, 0, "stone".to_string()).await;
        bm.update_block(-1, 0, 0, "stone".to_string()).await;
        bm.update_block(0, 1, 0, "stone".to_string()).await;
        bm.update_block(0, -1, 0, "stone".to_string()).await;
        bm.update_block(0, 0, 1, "stone".to_string()).await;
        bm.update_block(0, 0, -1, "stone".to_string()).await;
        
        let result = pl.reserve_path(1, from, to, 10);
        assert!(result.is_none());
    }
}