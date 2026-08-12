mod read_map;
mod a_star;
mod random_functions;
//mod a_estrella;
mod distances_types;
fn main(){
    let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    read_map::read_map(&mut array, "test_maps/arena2.map");
    //print!("{:?}", array[0]);
    //let res = a_star::a_star((0, 0), (5,5), &mut array, distances_types::euclidean_distance);
    let test_results = random_functions::test_astar(&mut array, 15);
    for (time, path) in test_results{
        println!("Time: {}ms", time);
        println!("path: ");
        println!("{:?}", path);
    }
}
