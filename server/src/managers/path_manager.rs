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
            block_manager: block_manager
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

                // 1. Static collision check
                if self.block_manager.get_block(neighbor.x, neighbor.y, neighbor.z).is_some() {
                    continue;
                }

                // 2. Dynamic collision check
                if let Some(res_id) = self.ledger.get(&coord) {
                    if *res_id != turtle_id {
                        continue;
                    }
                }

                // 3. Swap collision check (optional but recommended)
                // If we move from current to neighbor, check if someone is moving from neighbor to current
                let swap_coord = Coord::from((current, next_t));
                if let Some(res_id_at_neighbor_prev) = self.ledger.get(&Coord::from((neighbor, t))) {
                    if *res_id_at_neighbor_prev != turtle_id {
                        // Someone was at neighbor at t. 
                        // If they are at current at t+1, it's a swap.
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
        // Don't forget the start node if needed, but usually we only want the future steps
        // path.push(Coord::from(curr)); 
        path.reverse();

        if path.is_empty() && from != to {
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