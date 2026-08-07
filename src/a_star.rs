use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;
type Coords = (u32, u32);
type Distance = u64;


fn euclidean_distance_squared(tup1: Coords, tup2: Coords)->u64{
    return (tup1.0 + tup2.0).pow(2) as u64 +(tup1.1 + tup2.1).pow(2) as u64;
}

fn is_goal(best: &(Reverse<u64>, Coords))->bool{
    /* Something happends */
    return true;
}

pub fn a_star<Func>(start: Coords, goal: Coords, map:&[&[bool]], type_of_distance: Func) /*Hay que poner el return*/
where Func: Fn(Coords, Coords)->Distance
{
    let mut open: BinaryHeap<(Reverse<Distance>, Coords)> = BinaryHeap::new();
    let mut close: HashMap<Coords, Distance> = HashMap::new();
    open.push((Reverse(type_of_distance(start, goal)), start));
    while !open.is_empty() {
        let best  = open.peek().unwrap();
        if is_goal(best){
            return path;
        }
        let (val, key) = open.pop().unwrap();
        close.insert(key, val.0);
        for /*action to do */{

        }
    }
}