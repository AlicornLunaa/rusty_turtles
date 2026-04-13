/// This module handles path finding for robots
pub mod pathfinding {
    use std::cmp::Ordering;
    use std::collections::{BinaryHeap, HashMap};
    use tokio::time::Instant;

    use crate::{managers::block_manager::BlockManager, util::vector::Vector3};

    const MAX_NODES: usize = 100000;

    #[derive(Copy, Clone, Eq, PartialEq)]
    struct Node {
        position: Vector3,
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

    fn heuristic(a: Vector3, b: Vector3) -> i64 {
        Vector3::manhattan_distance(&a, &b)
    }

    pub async fn find_path(blocks: &BlockManager, start: Vector3, end: Vector3) -> Option<Vec<Vector3>> {
        let start_time = Instant::now();
        print!("Searching DB for path, ");

        let mut came_from: HashMap<Vector3, Vector3> = HashMap::new();
        let mut g_score: HashMap<Vector3, i64> = HashMap::new();
        let mut open_set = BinaryHeap::new();
        let mut nodes_explored = 0;
        
        g_score.insert(start, 0);
        open_set.push(Node { position: start, f_score: heuristic(start, end) });

        while let Some(Node { position: current, .. }) = open_set.pop() {
            // Prevent memory leak
            nodes_explored += 1;

            if nodes_explored > MAX_NODES {
                let elapsed = start_time.elapsed().as_secs_f64();
                println!("no path found, took {elapsed} seconds.");
                return None;
            }

            // Goal found
            if current == end {
                let mut path = vec![current];
                let mut curr = current;

                while let Some(&prev) = came_from.get(&curr) {
                    path.push(prev);
                    curr = prev;
                }

                path.reverse();

                let elapsed = start_time.elapsed().as_secs_f64();
                println!("found path in {elapsed} seconds.");

                return Some(path);
            }

            let neighbors = [
                Vector3::new(current.x + 1, current.y, current.z),
                Vector3::new(current.x - 1, current.y, current.z),
                Vector3::new(current.x, current.y + 1, current.z),
                Vector3::new(current.x, current.y - 1, current.z),
                Vector3::new(current.x, current.y, current.z + 1),
                Vector3::new(current.x, current.y, current.z - 1),
            ];

            for neighbor in neighbors {
                // Treat existing blocks as solid/impassable terrain
                if blocks.get_block(neighbor.x, neighbor.y, neighbor.z).is_some() {
                    continue;
                }

                let tentative_g_score = g_score[&current] + 1;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i64::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g_score);

                    let f_score = tentative_g_score + heuristic(neighbor, end);
                    open_set.push(Node { position: neighbor, f_score });
                }
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        println!("no path found, took {elapsed} seconds.");
        
        None
    }
}