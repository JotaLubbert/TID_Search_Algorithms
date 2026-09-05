mod a_star;
mod read_files;
use std::time::Instant;
use crate::read_files::{decode_scen, read_lines};
mod search_functions;
mod testing_functions;
mod write_files;
mod distances_types;
mod map_visualization;

type CustomMap = [[bool; 1024]; 1024];
fn main() {
    let mut array: CustomMap = [[false; 1024]; 1024];
    // read_files::read_map(&mut array, "maps/arena2.map");
    // let star_time = Instant::now();
    // let _ = testing_functions::test_visualizer(&mut array);
    // let finish = star_time.elapsed().as_millis();
    // print!("{}ms", finish)


    testing_functions::test_astar_correctnes(&mut array);


    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
