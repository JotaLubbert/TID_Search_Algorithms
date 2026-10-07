use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::mem::{align_of, size_of};

use crate::CustomMap;
use crate::closed_set::ClosedSet;

pub type Coords = (u32, u32);
type Distance = f64;
type OpenType = BinaryHeap<Reverse<SearchNode>>;
const COSTO_CARDINAL: f64 = 1.0;
const COSTO_DIAGONAL: f64 = std::f64::consts::SQRT_2;

const MOVEMENT_OPTIONS: [(i32, i32, f64); 8] = [
    (1, 0, COSTO_CARDINAL),
    (-1, 0, COSTO_CARDINAL),
    (0, -1, COSTO_CARDINAL),
    (0, 1, COSTO_CARDINAL),
    (-1, -1, COSTO_DIAGONAL),
    (-1, 1, COSTO_DIAGONAL),
    (1, -1, COSTO_DIAGONAL),
    (1, 1, COSTO_DIAGONAL),
];

#[derive(Clone, Copy, Debug)]
pub struct SearchNode {
    pub coords: Coords,
    pub g: Distance,
    pub _h: Distance,
    pub f: Distance,
    pub parent: Option<Coords>,
}
impl SearchNode {
    pub fn new(coords: Coords, g: f64, h: f64, parent: Option<Coords>) -> Self {
        Self { coords, g, _h: h, f: g + h, parent }
    }
}
impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool { self.f == other.f }
}
impl Eq for SearchNode {}
impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(other)) }
}
impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.f.total_cmp(&other.f) }
}

pub struct AStarResults {
    pub final_dis: Distance,
    pub path: Vec<Coords>,
    pub open: OpenType,
    pub open_bytes: usize,
    pub close_bytes: usize,
    pub expansions: u64,
    pub generated: u64,
    pub open_nodes: usize,
    pub close_nodes: usize,
    pub open_node_bytes: usize,
    pub close_node_bytes: usize,
}

const GROUP_WIDTH: usize =
    if cfg!(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
    )) {
        16
    } else if cfg!(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "little",
    )) {
        8
    } else {
        size_of::<usize>()
    };

fn open_size_in_bytes(open: &BinaryHeap<Reverse<SearchNode>>) -> usize {
    size_of::<BinaryHeap<Reverse<SearchNode>>>()
        + open.capacity() * size_of::<Reverse<SearchNode>>()
}

fn buckets_for(capacity: usize) -> usize {
    if capacity < 8 {
        if capacity < 4 { 4 } else { 8 }
    } else {
        (capacity * 8 / 7).next_power_of_two()
    }
}

// pública: la usa el impl de ClosedSet para HashMap
pub fn close_size_in_bytes(close: &HashMap<Coords, SearchNode>) -> usize {
    let base = size_of::<HashMap<Coords, SearchNode>>();
    if close.capacity() == 0 {
        return base;
    }
    let buckets = buckets_for(close.capacity());
    let entry = size_of::<(Coords, SearchNode)>();
    let ctrl_align = align_of::<(Coords, SearchNode)>().max(GROUP_WIDTH);
    let ctrl_offset = (buckets * entry).next_multiple_of(ctrl_align);
    base + ctrl_offset + buckets + GROUP_WIDTH
}

fn reconstruct_path<C: ClosedSet>(close: &C, mut current: Coords) -> Vec<Coords> {
    let mut path = vec![current];
    while let Some(parent) = close.get(current).unwrap().parent {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    path
}

fn valid_succesors(current_coords: Coords, map: &CustomMap) -> (Vec<Coords>, Vec<f64>) {
    let mut posible_moves: Vec<Coords> = vec![];
    let mut movement_cost: Vec<f64> = vec![];
    for (x, y, cost) in MOVEMENT_OPTIONS {
        let can_operate_x = current_coords.0 > 0 || x != -1;
        let can_operate_y = current_coords.1 > 0 || y != -1;
        if !can_operate_x || !can_operate_y {
            continue;
        }
        let search_x = (current_coords.0 as i32 + x) as u32;
        let search_y = (current_coords.1 as i32 + y) as u32;
        if search_x as usize >= map[0].len() || search_y as usize >= map.len() {
            continue;
        }
        let can_go_there = map[search_y as usize][search_x as usize];
        if !can_go_there {
            continue;
        }
        // evita el corner cutting
        if x != 0 && y != 0 {
            if !map[(search_y as i32 - y) as usize][search_x as usize]
                || !map[search_y as usize][(search_x as i32 - x) as usize]
            {
                continue;
            }
        }
        posible_moves.push((search_x, search_y));
        movement_cost.push(cost);
    }
    return (posible_moves, movement_cost);
}

pub fn a_star<C: ClosedSet, Func>(
    start: Coords,
    goal: Coords,
    map: &CustomMap,
    type_of_distance: Func,
) -> Option<AStarResults>
where
    Func: Fn(Coords, Coords) -> Distance,
{
    if !map[start.1 as usize][start.0 as usize] {
        return None;
    }

    let mut open: OpenType = BinaryHeap::new();
    let mut close = C::new();

    let h_start = type_of_distance(start, goal);
    let start_node = SearchNode::new(start, 0.0, h_start, None);
    close.insert(start_node);
    open.push(Reverse(start_node));
    let mut expansions: u64 = 0;
    let mut generated: u64 = 0;
    while let Some(Reverse(current)) = open.pop() {
        let current_g = close.get(current.coords).unwrap().g;
        if current.g > current_g {
            continue;
        }
        expansions += 1;
        if current.coords == goal {
            let path = reconstruct_path(&close, current.coords);
            let open_bytes = open_size_in_bytes(&open);
            let close_bytes = close.size_in_bytes();
            let open_nodes = open.len();
            let close_nodes = close.len();
            let open_node_bytes = open_nodes * size_of::<Reverse<SearchNode>>();
            let close_node_bytes = close_nodes * size_of::<SearchNode>();
            return Some(AStarResults {
                final_dis: current_g,
                path,
                open,
                open_bytes,
                close_bytes,
                expansions,
                generated,
                open_nodes,
                close_nodes,
                open_node_bytes,
                close_node_bytes,
            });
        }

        let (position_succesor, weight) = valid_succesors(current.coords, map);

        for (neighbor, cost) in position_succesor.iter().zip(weight.iter()) {
            let tentative_g = current_g + *cost;
            let existing_g = close.get(*neighbor).map(|n| n.g).unwrap_or(f64::INFINITY);

            if tentative_g < existing_g {
                let h = type_of_distance(*neighbor, goal);
                let neighbor_node = SearchNode::new(*neighbor, tentative_g, h, Some(current.coords));
                close.insert(neighbor_node);
                generated += 1;
                open.push(Reverse(neighbor_node));
            }
        }
    }
    return None;
}
