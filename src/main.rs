mod read_map;
mod a_star;
mod a_estrella;
mod distances_types;
fn main(){
    let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    read_map::read_map(&mut array, "test_maps/arena2.map");
    print!("{:?}", array[2])
    a_star::a_star((0, 0), (5,5), array, distances_types::euclidean_distance);
}
