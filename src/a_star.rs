use std::collections::BinaryHeap;
use std::collections::HashMap;

fn euclidean_distance(vec1: &[f64], vec2: &[f64])->f64{
    if vec1.iter().len() != vec2.iter().len() {
        panic!("Expected 2 equal size vectors");
    }
    let mut res: f64 = 0.0;
    for (i, val) in vec1.iter().enumerate() {
        res += (vec2[i] - val).powi(2);
    }
    return res;
}

pub fn a_star<Func>(map:&[&[bool]], distance: Func)
where Func: Fn(&[f64], &[f64])->f64
{
    let mut open: BinaryHeap<(usize, usize)> = BinaryHeap::new();
    let mut close = HashMap::new();
}