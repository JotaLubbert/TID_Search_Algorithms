type Coords = (u32, u32);
type Distance = f64;

pub fn euclidean_distance(tup1: Coords, tup2: Coords) -> Distance {
    let dx = tup1.0 as f64 - tup2.0 as f64;
    let dy = tup1.1 as f64 - tup2.1 as f64;
    return (dx * dx + dy * dy).sqrt();
}


pub fn manhattan_distance(tup1: Coords, tup2: Coords)->Distance{
    let dx = (tup1.0 as f64 - tup2.0 as f64).abs();
    let dy = (tup1.1 as f64 - tup2.1 as f64).abs();
    return dx + dy;
}
