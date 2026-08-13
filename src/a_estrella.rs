use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

type Coords = (u32, u32);
type Distance = f64;

const COSTO_CARDINAL: Distance = 1.0;
const COSTO_DIAGONAL: Distance = std::f64::consts::SQRT_2;

// recordar, que me pasó que se me olvidó que al ser matriz es +1 hacia abajo y -1 hacia arriba
const DIRECTIONS: [(i32, i32, Distance); 8] = [
    (1, 0, COSTO_CARDINAL), // abajo
    (-1, 0, COSTO_CARDINAL), // arriba
    (0, -1, COSTO_CARDINAL), // izquierda
    (0, 1, COSTO_CARDINAL), // derecha
    (-1, -1, COSTO_DIAGONAL), // diagonal superior-izquierda
    (-1, 1, COSTO_DIAGONAL), // diagonal superior-derecha
    (1, -1, COSTO_DIAGONAL), // diagonal inferior-izquierda
    (1, 1, COSTO_DIAGONAL), // diagonal inferior-derecha
];

fn euclidean_distance(a: Coords, b: Coords) -> Distance {
    let dr = a.0.abs_diff(b.0) as Distance;
    let dc = a.1.abs_diff(b.1) as Distance;
    (dr * dr + dc * dc).sqrt()
}

// en teoría, TEORÍA, siempre deberiamos tener los puntos inicial y final, lo digo porque ví tu función de is_goal y no entiendo muy bien que hará xD
pub fn astar<Func>(start: Coords, goal: Coords, map: &[[bool; 512]; 512], heuristic: Func) -> Option<(Vec<Coords>, Distance)>
where Func: Fn(Coords, Coords) -> Distance, {
    
    let mut open: BinaryHeap<(Reverse<u64>, Coords)> = BinaryHeap::new();
    let mut best_g: HashMap<Coords, Distance> = HashMap::new();
    let mut closed: HashSet<Coords> = HashSet::new();
    let mut parent: HashMap<Coords, Coords> = HashMap::new();

    best_g.insert(start, 0.0);    
    open.push((Reverse(heuristic(start, goal).to_bits()), start));

    while let Some((_, best)) = open.pop() {
        if closed.contains(&best) {
            continue;
        }

        if best == goal {
            let mut path = vec![best];
            let mut cur = best;
            while let Some(&p) = parent.get(&cur) {
                path.push(p);
                cur = p;
            }
            path.reverse();
            return Some((path, best_g[&best]));
        }
        closed.insert(best);
        let g_best = best_g[&best];

        for &(dr, dc, base_cost) in &DIRECTIONS {
            let nr = best.0 as i32 + dr;
            let nc = best.1 as i32 + dc;
            if nr < 0 || nr >= map.len() as i32 || nc < 0 || nc >= map[0].len() as i32 {
                continue;
            }
            
            let child: Coords = (nr as u32, nc as u32);
            if !map[child.0 as usize][child.1 as usize] {
                continue;
            }

            if closed.contains(&child) {
                continue;
            }

            let g_new = g_best + base_cost;
            if let Some(&g_old) = best_g.get(&child) {
                if g_old <= g_new {
                    continue;
                }
            }
                
            best_g.insert(child, g_new);
            parent.insert(child, best);
            let f = g_new + heuristic(child, goal);
            open.push((Reverse(f.to_bits()), child));
        }
    }
    None // por si acaso no hay camino aunque no debería pasar
}
