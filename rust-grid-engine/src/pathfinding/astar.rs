use crate::grid::neighbours_4 as neigh;
use crate::grid::{Dir, GridCoord};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

#[derive(Clone)]
pub struct AStarPolicy {
    pub passable: Arc<dyn Fn(GridCoord) -> bool + Send + Sync>,
    pub cost: Arc<dyn Fn(GridCoord, GridCoord) -> u32 + Send + Sync>,
}

//the estimated cost from a to b
fn manhattan(a: GridCoord, b: GridCoord) -> u32 {
    (a.x - b.x).abs() as u32 + (a.y - b.y).abs() as u32
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    pos: GridCoord,
    f: u32,
    g: u32,
}
// Min-heap based on f = g + h
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn astar(start: GridCoord, goal: GridCoord, policy: &AStarPolicy) -> Option<Vec<GridCoord>> {
    let mut open = BinaryHeap::new();
    let mut came: HashMap<GridCoord, GridCoord> = HashMap::new();
    let mut g: HashMap<GridCoord, u32> = HashMap::new();

    g.insert(start, 0);
    open.push(Node {
        pos: start,
        f: manhattan(start, goal),
        g: 0,
    });

    while let Some(Node { pos, g: gscore, .. }) = open.pop() {
        if pos == goal {
            // reconstruct
            let mut path = vec![pos];
            let mut cur = pos;
            while let Some(&p) = came.get(&cur) {
                path.push(p);
                cur = p;
            }
            path.reverse();
            return Some(path);
        }
        for n in neigh(pos) {
            if !(policy.passable)(n) {
                continue;
            }
            let tentative = gscore + (policy.cost)(pos, n);
            if tentative < *g.get(&n).unwrap_or(&u32::MAX) {
                came.insert(n, pos);
                g.insert(n, tentative);
                let f = tentative + manhattan(n, goal);
                open.push(Node {
                    pos: n,
                    f,
                    g: tentative,
                });
            }
        }
    }
    None
}
