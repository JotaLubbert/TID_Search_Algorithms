use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use ordered_float::OrderedFloat;
type Coords = (u32, u32);
type Distance = f64;

const costo_cardinal: f64 = 1.0;
const costo_diagonal: f64 = std::f64::consts::SQRT_2;

const movement_options: [(i32, i32, f64); 8] = [
    (1, 0, costo_cardinal), // abajo
    (-1, 0, costo_cardinal), // arriba
    (0, -1, costo_cardinal), // izquierda
    (0, 1, costo_cardinal), // derecha
    (-1, -1, costo_diagonal), // diagonal superior-izquierda
    (-1, 1, costo_diagonal), // diagonal superior-derecha
    (1, -1, costo_diagonal), // diagonal inferior-izquierda
    (1, 1, costo_diagonal), // diagonal inferior-derecha
];

fn is_in_heap(heap: &BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)>, tuple: Coords)->bool{
    for (_distance, coords) in heap {
        if *coords == tuple{
            return true;
        }
    }
    return false;
}

pub fn a_star<Func>(start: Coords, goal: Coords, map:&[&[bool]], type_of_distance: Func)->(Distance, Vec<Coords>)
where Func: Fn(Coords, Coords)->Distance
{
    let mut final_cost: f64 = 0.0;
    let mut open: BinaryHeap<(Reverse<OrderedFloat<Distance>>, Coords)> = BinaryHeap::new();
    let mut close: HashMap<Coords, Distance> = HashMap::new();
    let mut f_score = type_of_distance(start, goal);
    open.push((Reverse(OrderedFloat(f_score)), start));
    let mut path = vec![];
    while !open.is_empty() {
        let (val, best)  = open.pop().unwrap();
        if best == goal{
            return (final_cost, path);
        }
        let val = val.0;
        close.insert(best, val.0);
        for (x, y, cost) in movement_options{
            let can_operate_x = best.0 > 0 || x != -1;
            let can_operate_y = best.1 > 0 || y != -1;
            if !can_operate_x || !can_operate_y{
                continue;
            }
            let search_x = (best.0 as i32 + x) as u32;
            let search_y = (best.1 as i32 + y) as u32;
            let can_go_there = !map[search_y as usize][search_x as usize];
            if !can_go_there {
                continue;
            }
            let tuple_search = (search_x, search_y);
            let cost_in_node = final_cost + cost;
            if close.contains_key(&tuple_search) || is_in_heap(&open, tuple_search){
                if final_cost <= cost_in_node{
                    continue;
                }
                
            }
            final_cost += cost;
            f_score = final_cost + type_of_distance(tuple_search, goal);
            open.push((Reverse(OrderedFloat(f_score)), tuple_search));
            path.push(tuple_search);
        }
    }
    return (final_cost, path);
}