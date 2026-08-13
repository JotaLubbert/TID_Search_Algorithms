mod read_map;
mod a_star;
mod random_functions;
//mod a_estrella;
mod distances_types;
fn main(){
    let mut vector: Vec<Vec<bool>> = vec![Vec::new()];
    read_map::read_map(&mut vector, "test_maps/arena2.map");
    //print!("{:?}", array[0]);
    //let res = a_star::a_star((0, 0), (5,5), &mut array, distances_types::euclidean_distance);
    let test_results = random_functions::test_astar(&vector, 1);
    for (time, path) in test_results{
        println!("Time: {}ms", time);
        println!("path: ");
        println!("{:?}", path);
    }
}
