use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::mem::{align_of, size_of};

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
    pub _h: Distance,
    pub f: Distance,
    pub parent: Option<Coords>,
}
impl SearchNode {
    //El nodo ahora tiene está en el heap, por lo que necesita sus coordenadas, f es cambiado por g + h
    pub fn new(coords: Coords, g: f64, h: f64, parent: Option<Coords>) -> Self {
        Self { coords, g, _h: h, f: g + h, parent }
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

pub struct AStarResults{
    pub final_dis: Distance,
    pub path: Vec<Coords>,
    //el open y el close ya no se devuelven completos, solo lo que ocupan en memoria
    pub open_bytes: usize,
    pub close_bytes: usize,
    pub expansions: u64,
    pub generated: u64,
}

//Ancho del grupo que usa hashbrown (la tabla detrás de HashMap) para escanear
//control bytes. Depende de la arquitectura, así que replicamos su misma
//selección de cfg: SSE2 en x86 escanea de a 16, NEON en aarch64 de a 8
//(uint8x8_t) y el resto cae al fallback genérico, que escanea de a un usize.
const GROUP_WIDTH: usize =
    if cfg!(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse2",
    )) {
        16
    } else if cfg!(all(
        target_arch = "aarch64",
        target_feature = "neon",
        //hashbrown evita NEON en big-endian porque los intrínsecos fallan ahí
        target_endian = "little",
    )) {
        8
    } else {
        size_of::<usize>()
    };

//size_of_val no sirve acá: solo mide el struct en el stack y el buffer del heap
//queda fuera, que es justamente lo que crece durante la búsqueda.
fn open_size_in_bytes(open: &BinaryHeap<Reverse<SearchNode>>) -> usize {
    size_of::<BinaryHeap<Reverse<SearchNode>>>()
        + open.capacity() * size_of::<Reverse<SearchNode>>()
}

//hashbrown reserva una potencia de 2 de buckets y mantiene ocupación máxima de 7/8
fn buckets_for(capacity: usize) -> usize {
    if capacity < 8 {
        if capacity < 4 { 4 } else { 8 }
    } else {
        (capacity * 8 / 7).next_power_of_two()
    }
}

fn close_size_in_bytes(close: &HashMap<Coords, SearchNode>) -> usize {
    let base = size_of::<HashMap<Coords, SearchNode>>();
    if close.capacity() == 0 {
        //un mapa sin capacidad todavía no pide memoria al allocator
        return base;
    }
    let buckets = buckets_for(close.capacity());
    let entry = size_of::<(Coords, SearchNode)>();
    //los bytes de control van después del arreglo de entradas, realineados
    let ctrl_align = align_of::<(Coords, SearchNode)>().max(GROUP_WIDTH);
    let ctrl_offset = (buckets * entry).next_multiple_of(ctrl_align);
    base + ctrl_offset + buckets + GROUP_WIDTH
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
        let can_go_there = map[search_y as usize][search_x as usize];
        if !can_go_there {
            continue;
        }
        //evita el corner cutting
        if x != 0 && y != 0 {
            if !map[(search_y as i32 - y) as usize][search_x as usize]
                || !map[search_y as usize][(search_x as i32 - x) as usize] //hay que revisar esto, si es o un Y
            {
                continue;
            }
        }
        posible_moves.push((search_x, search_y));
        movement_cost.push(cost);
    }
    return (posible_moves, movement_cost);
}


//Lo mismo, no necesita mutabilidad
pub fn a_star<Func>(
    start: Coords,
    goal: Coords,
    map: &CustomMap,
    type_of_distance: Func,
) -> Option<
    AStarResults
>
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
    let mut expansions: u64 = 0;
    let mut generated: u64 = 0;
    while let Some(Reverse(current)) = open.pop() {
        
        let current_g = close[&current.coords].g;
        //Se compara de forma directa el g acutal con el mejor g encontrado en el close
        if current.g > current_g {
            continue;
        }
        expansions += 1;
        if current.coords == goal {
            return Some(AStarResults{final_dis: current_g,
                path: reconstruct_path(&close, current.coords),
                open_bytes: open_size_in_bytes(&open),
                close_bytes: close_size_in_bytes(&close),
                expansions: expansions,
                generated: generated

            });
        }

        let (position_succesor, weight) = valid_succesors(current.coords, map);

        for (neighbor, cost) in position_succesor.iter().zip(weight.iter()) {
            let tentative_g = current_g + *cost;
            let existing_g = close.get(neighbor).map(|n| n.g).unwrap_or(f64::INFINITY);

            if tentative_g < existing_g {
                let h = type_of_distance(*neighbor, goal);
                let neighbor_node = SearchNode::new(*neighbor, tentative_g, h, Some(current.coords));
                close.insert(*neighbor, neighbor_node);
                generated += 1;
                open.push(Reverse(neighbor_node));
            }
        }
    }
    return None
}
