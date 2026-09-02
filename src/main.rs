mod a_star;
mod read_map;
use std::time::Instant;
mod search_functions;
mod testing_functions;
//mod a_estrella;
mod distances_types;
mod map_visualization;
fn main() {
    let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    read_map::read_map(&mut array, "test_maps/arena2.map");
    let star_time = Instant::now();
    let _ = testing_functions::test_visualizer(&mut array);
    let finish = star_time.elapsed().as_millis();
    print!("{}ms", finish)
    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
