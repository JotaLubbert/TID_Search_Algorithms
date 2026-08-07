use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;

type Coords = (u32, u32);
type Distance = f64;

const costo_cardinal: Distance = 1.0;
const costo_diagonal: Distance = std::f64::consts::SQRT_2;

// recordar, que me pasó que se me olvidó que al ser matriz es +1 hacia abajo y -1 hacia abajo
const DIRECTIONS: [(i32, i32, Distance); 8] = [
    (1, 0, costo_cardinal), // abajo
    (-1, 0, costo_cardinal), // arriba
    (0, -1, costo_cardinal), // izquierda
    (0, 1, costo_cardinal), // derecha
    (-1, -1, costo_diagonal), // diagonal superior-izquierda
    (-1, 1, costo_diagonal), // diagonal superior-derecha
    (1, -1, costo_diagonal), // diagonal inferior-izquierda
    (1, 1, costo_diagonal), // diagonal inferior-derecha
];

fn euclidean_distance(a: Coords, b: Coords) -> Distance {
    let dr = a.0.abs_diff(b.0) as Distance;
    let dc = a.1.abs_diff(b.1) as Distance;
    (dr * dr + dc * dc).sqrt()
}

// en teoría, TEORÍA, siempre deberiamos tener los puntos inicial y final, lo digo porque ví tu función de is_goal y no entiendo muy bien que hará xD
pub fn astar<Func>(start: Coords, goal: Coords, map: &[&[bool]], type_of_distance: Func) -> Option<(Vec<Coords>, Distance)>
where Func: Fn(Coords, Coords) -> Distance,
{
    let mut open: BinaryHeap<(Reverse<u64>, Coords)> = BinaryHeap::new();
    let mut close: HashMap<Coords, Distance> = HashMap::new();
    open.push((Reverse(type_of_distance(start, goal).to_bits()), start));

    while !open.is_empty() {
        let &(_, best) = open.peek().unwrap();
        if best == goal {
            return None;
        }

        let (Reverse(g_bits), key) = open.pop().unwrap();
        let g = Distance::from_bits(g_bits);
        close.insert(key, g);

        for &(dr, dc, base_cost) in &DIRECTIONS {
            let nr = key.0 as i32 + dr;
            let nc = key.1 as i32 + dc;
 
            if nr < 0 || nr >= map.len() as i32 || nc < 0 || nc >= map[0].len() as i32 {
                continue;
            }
            let child: Coords = (nr as u32, nc as u32);

            if !map[child.0 as usize][child.1 as usize] {
                continue;
            }
        }
    }

    None // por si acaso no hay camino aunque no debería pasar
}
