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


fn show_open_close_size(open: &BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)>, close: &HashMap<Coords, Distance>){
    let open_entry_size = size_of::<(Reverse<OrderedFloat<Distance>>, Coords)>();
    let close_entry_size = size_of::<(Coords, Distance)>(); // key + value aproximado

    let total_open_size = open.capacity() * open_entry_size;
    let total_close_size = close.capacity() * close_entry_size;

    println!("Tamaño del open: {} bytes ({} entradas, capacidad {})", total_open_size, open.len(), open.capacity());
    println!("Tamaño del close: {} bytes ({} entradas, capacidad {})", total_close_size, close.len(), close.capacity()); // Aproximación del tamaño de la memoria utilizada por el HashMap
}


fn reconstruct_path(came_from: &HashMap<Coords, Coords>, mut current: Coords) -> Vec<Coords> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    return path;
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
        if x != 0 && y != 0{
            if !map[(search_y as i32 - y) as usize][search_x as usize] && !map[search_y as usize][(search_x as i32 - x) as usize]{
                continue;
            }
        }
        posible_moves.push((search_x, search_y));
        movement_cost.push(cost);
    }
    return (posible_moves, movement_cost);
}

pub fn a_star<Func>(start: Coords, goal: Coords, map:&mut[[bool; 512]; 512], type_of_distance: Func)->Option<(Distance, Vec<Coords>)>
where Func: Fn(Coords, Coords)->Distance
{
    if !map[start.1 as usize][start.0 as usize]{
        return None;
    }
    let mut open: BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)> = BinaryHeap::new();
    let mut close: HashMap<Coords, Distance> = HashMap::new();
    let mut g_score: HashMap<Coords, Distance> = HashMap::new();
    let mut path: HashMap<Coords, Coords> = HashMap::new();
    g_score.insert(start, 0.0);
    let f_score = type_of_distance(start, goal);
    open.push((Reverse(OrderedFloat(f_score)), start));

    let mut expansions: u64 = 0;
    // Reverse y OrderedFloats son solo wrapers para que la función funcione, no añaden complejidad adicional.
    while let Some((Reverse(OrderedFloat(f)), current)) = open.pop() {
        let current_g = g_score[&current];
        let current_f = current_g + type_of_distance(current, goal);
        if f > current_f {
            // Si entra a la linea 91
            continue;
        }
        
        //mostrar las expanciones
        expansions += 1;
        println!("Expansión #{}: nodo {:?}, g={:.2}, f={:.2}", expansions, current, current_g, current_f);

        if current == goal{
            // mostrar tamaño del open y close en bytes
            // show_open_close_size(&open, &close);
            return Some((current_g, reconstruct_path(&path, current)));
        }

        close.insert(current, current_g);

        let (position_succesor, weight) = valid_succesors(current, map);

        let mut generated: u64 = 0;
        
        for (neighbor, cost) in position_succesor.iter().zip(weight.iter()){
            let tentative_g = current_g + *cost;
            if let Some(&closed_g) = close.get(neighbor){
                if closed_g <= tentative_g{
                    // También entra a la linea 107. no son redundantes.
                    continue;
                }
                close.remove(&neighbor);
            }
            let existing_g = g_score.get(&neighbor).copied().unwrap_or(f64::INFINITY);
            if tentative_g < existing_g {
                path.insert(*neighbor, current);
                g_score.insert(*neighbor, tentative_g);
                let f_neighbor = tentative_g + type_of_distance(*neighbor, goal);
                open.push((Reverse(OrderedFloat(f_neighbor)), *neighbor));
                
                // mostrar estados generados por el for.
                generated += 1;
                println!("  Estado generado #{}: {:?}, g={:.2}, f={:.2}", generated, neighbor, tentative_g, f_neighbor);
            }
        }
    }
    return None;
}
