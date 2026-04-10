/// This module handles path finding for robots
mod pathfinding {
    use std::cmp::Ordering;
    use std::collections::{BinaryHeap, HashMap};
    use crate::{managers::block_manager::BlockManager, util::vector::Vector3};

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
        let mut came_from: HashMap<Vector3, Vector3> = HashMap::new();
        let mut g_score: HashMap<Vector3, i64> = HashMap::new();
        let mut open_set = BinaryHeap::new();
        
        g_score.insert(start, 0);
        open_set.push(Node { position: start, f_score: heuristic(start, end) });

        while let Some(Node { position: current, .. }) = open_set.pop() {
            if current == end {
                let mut path = vec![current];
                let mut curr = current;

                while let Some(&prev) = came_from.get(&curr) {
                    path.push(prev);
                    curr = prev;
                }

                path.reverse();
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
                if blocks.get_block(neighbor.x, neighbor.y, neighbor.z).await.is_some() {
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
        
        None
    }
}