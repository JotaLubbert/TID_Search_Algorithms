mod read_map;
mod a_star;
mod search_functions;
mod testing_functions;
//mod a_estrella;
mod distances_types;
mod map_visualization;
fn main(){
    let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    read_map::read_map(&mut array, "test_maps/arena2.map");
    //print!("{:?}", array[0]);
    //let res = a_star::a_star((0, 0), (5,5), &mut array, distances_types::euclidean_distance);
    // let test_results = testing_functions::test_astar(&mut array, 15);
    // for (time, path) in test_results{
    //     println!("Time: {}ms", time);
    //     println!("path: ");
    //     println!("{:?}", path);
    // }
    testing_functions::test_visualizer(&mut array);

}
