use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::CustomMap;

pub type Coords = (u32, u32);
type Distance = f64;

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
    pub h: Distance,
    pub f: Distance,
    pub parent: Option<Coords>,
}
impl SearchNode {
    //El nodo ahora tiene está en el heap, por lo que necesita sus coordenadas, f es cambiado por g + h
    pub fn new(coords: Coords, g: f64, h: f64, parent: Option<Coords>) -> Self {
        Self { coords, g, h, f: g + h, parent }
    }
}
//Implementación del ord, el ord permite decidir la prioridad de los nofos
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

fn reconstruct_path(close: &HashMap<Coords, SearchNode>, mut current: Coords) -> Vec<Coords> {
    let mut path = vec![current];
    while let Some(parent) = close[&current].parent {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    path
}

//Ahora no recibe el mapa como mutable, no cambia
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
        if !map[search_y as usize][search_x as usize] {
            continue;
        }
        posible_moves.push((search_x, search_y));
        movement_cost.push(cost);
    }
    return (posible_moves, movement_cost)
}

//Lo mismo, no necesita mutabilidad
pub fn a_star<Func>(
    start: Coords,
    goal: Coords,
    map: &CustomMap,
    type_of_distance: Func,
) -> Option<(
    Distance,
    Vec<Coords>,
    BinaryHeap<Reverse<SearchNode>>,
    HashMap<Coords, SearchNode>,
)>
where Func: Fn(Coords, Coords) -> Distance
{
    if !map[start.1 as usize][start.0 as usize] {
        return None;
    }

    let mut open: BinaryHeap<Reverse<SearchNode>> = BinaryHeap::new();
    //El heap ahora contiene nodos y no tuplas
    let mut close: HashMap<Coords, SearchNode> = HashMap::new();

    let h_start = type_of_distance(start, goal);
    let start_node = SearchNode::new(start, 0.0, h_start, None);
    close.insert(start, start_node);
    open.push(Reverse(start_node));

    while let Some(Reverse(current)) = open.pop() {
        
        let current_g = close[&current.coords].g;
        //Se compara de forma directa el g acutal con el mejor g encontrado en el close
        if current.g > current_g {
            continue;
        }

        if current.coords == goal {
            // mostrar tamaño del open y close en bytes
            // show_open_close_size(&open, &close);
            return Some((current_g, reconstruct_path(&close, current.coords), open, close));
        }

        let (position_succesor, weight) = valid_succesors(current.coords, map);

        for (neighbor, cost) in position_succesor.iter().zip(weight.iter()) {
            let tentative_g = current_g + *cost;
            let existing_g = close.get(neighbor).map(|n| n.g).unwrap_or(f64::INFINITY);

            if tentative_g < existing_g {
                let h = type_of_distance(*neighbor, goal);
                let neighbor_node = SearchNode::new(*neighbor, tentative_g, h, Some(current.coords));
                close.insert(*neighbor, neighbor_node);
                open.push(Reverse(neighbor_node));
            }
        }
    }
    return None
}
