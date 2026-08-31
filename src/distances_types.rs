type Coords = (u32, u32);
type Distance = f64;

pub fn euclidean_distance(tup1: Coords, tup2: Coords) -> Distance {
    return (((tup1.0 + tup2.0).pow(2) + (tup1.1 + tup2.1).pow(2)) as f64).powf(0.5);
}


pub fn manhattan_distance(tup1: Coords, tup2: Coords)->Distance{
    return (tup1.0 + tup2.0 + tup1.1 + tup2.1) as f64;
}
