use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use ordered_float::OrderedFloat;
type Coords = (u32, u32);
type Distance = f64;

const COSTO_CARDINAL: f64 = 1.0;
const COSTO_DIAGONAL: f64 = std::f64::consts::SQRT_2;

const MOVEMENT_OPTIONS: [(i32, i32, f64); 8] = [
    (1, 0, COSTO_CARDINAL), // abajo
    (-1, 0, COSTO_CARDINAL), // arriba
    (0, -1, COSTO_CARDINAL), // izquierda
    (0, 1, COSTO_CARDINAL), // derecha
    (-1, -1, COSTO_DIAGONAL), // diagonal superior-izquierda
    (-1, 1, COSTO_DIAGONAL), // diagonal superior-derecha
    (1, -1, COSTO_DIAGONAL), // diagonal inferior-izquierda
    (1, 1, COSTO_DIAGONAL), // diagonal inferior-derecha
];


// Se usa un almacenamiento de los padres y se actualizan
// Crear estructura de search node
// por lo general se usan arrays

#[derive(Clone, Copy, Debug)]
struct SearchNode {
    pub g: Distance,
    #[allow(dead_code)]
    pub h: Distance,
    #[allow(dead_code)]
    pub f: Distance,
    pub parent: Option<Coords>,
}
impl SearchNode {
    pub fn new(g: f64, h: f64, f: f64, parent: Option<(u32, u32)>)->Self{
        Self{g, h, f, parent}
    }
}

fn show_open_close_size(open: &BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)>, close: &HashMap<Coords, SearchNode>) {
    let open_entry_size = size_of::<(Reverse<OrderedFloat<Distance>>, Coords)>();
    let close_entry_size = size_of::<(Coords, SearchNode)>(); // key + value aproximado

    let total_open_size = open.capacity() * open_entry_size;
    let total_close_size = close.capacity() * close_entry_size;

    println!("Tamaño del open: {} bytes ({} entradas, capacidad {})", total_open_size, open.len(), open.capacity());
    println!("Tamaño del close: {} bytes ({} entradas, capacidad {})", total_close_size, close.len(), close.capacity()); // Aproximación del tamaño de la memoria utilizada por el HashMap
}

fn reconstruct_path(node: &HashMap<Coords, SearchNode>, mut current: Coords) -> Vec<Coords> {
    let mut path = vec![current];
    while let Some(parent) = node[&current].parent {
        path.push(parent);
        current = parent;
    }
    path.reverse();
    path
}

fn valid_succesors(current_coords: Coords, map:&mut[[bool; 512]; 512])->(Vec<Coords>, Vec<f64>){
    let mut posible_moves: Vec<Coords> = vec![];
    let mut movement_cost: Vec<f64> = vec![];
    for (x, y, cost) in MOVEMENT_OPTIONS{
        let can_operate_x = current_coords.0 > 0 || x != -1;
        let can_operate_y = current_coords.1 > 0 || y != -1;
        if !can_operate_x || !can_operate_y{
            continue;
        }
        let search_x = (current_coords.0 as i32 + x) as u32;
        let search_y = (current_coords.1 as i32 + y) as u32;
        if search_x as usize >= 512 || search_y as usize >= 512 {
            continue;
        }
        let can_go_there = map[search_y as usize][search_x as usize];
        if !can_go_there {
            continue;
        }
        posible_moves.push((search_x, search_y));
        movement_cost.push(cost);
    }
    return (posible_moves, movement_cost);
}

pub fn a_star<Func>(start: Coords, goal: Coords, map: &mut [[bool; 512]; 512], type_of_distance: Func) -> Option<(Distance, Vec<Coords>)>
where Func: Fn(Coords, Coords) -> Distance
{
    if !map[start.1 as usize][start.0 as usize] {
        return None;
    }
    
    let mut open: BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)> = BinaryHeap::new();
    let mut close: HashMap<Coords, SearchNode> = HashMap::new();
    
    let h_start = type_of_distance(start, goal);
    close.insert(start, SearchNode::new(0.0, h_start, h_start, None));
    open.push((Reverse(OrderedFloat(h_start)), start));
    
    let mut expansions: u64 = 0;
    
    while let Some((Reverse(OrderedFloat(f)), current)) = open.pop() {
        let current_g = close[&current].g;
        
        let current_f = current_g + type_of_distance(current, goal);
        if f > current_f {
            continue;
        }
        
        expansions += 1;
        println!("Expansión #{}: nodo {:?}, g={:.2}, f={:.2}", expansions, current, current_g, current_f);
        
        if current == goal {
            show_open_close_size(&open, &close);
            return Some((current_g, reconstruct_path(&close, current)));
        }

        let (position_succesor, weight) = valid_succesors(current, map);

        let mut generated: u64 = 0;
        for ((search_x, search_y), cost) in position_succesor.iter().zip(weight.iter()) {
            let neighbor = (*search_x, *search_y);
            let tentative_g = current_g + *cost;

            let existing_g = close.get(&neighbor).map(|n| n.g).unwrap_or(f64::INFINITY);

            if tentative_g < existing_g {
                let h = type_of_distance(neighbor, goal);
                let f_neighbor = tentative_g + h;
                close.insert(neighbor, SearchNode::new(tentative_g, h, f_neighbor, Some(current)));
                open.push((Reverse(OrderedFloat(f_neighbor)), neighbor));

                generated += 1;
                println!("  Estado generado #{}: {:?}, g={:.2}, f={:.2}", generated, neighbor, tentative_g, f_neighbor);
            }
        }
    }
    return None;
}