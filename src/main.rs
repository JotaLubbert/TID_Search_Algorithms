mod a_star;
mod read_files;
use std::time::Instant;
use crate::read_files::{decode_scen, read_lines};
mod search_functions;
mod testing_functions;
mod write_files;
mod distances_types;
mod map_visualization;
mod closed_set;
mod robin_hood;
mod judy;
use std::collections::HashMap;
use crate::a_star::{Coords, SearchNode};

type CustomMap = [[bool; 2048]; 2048];
fn main() {
    let mut array: Box<CustomMap> = vec![[false; 2048]; 2048].into_boxed_slice().try_into().unwrap();
    // read_files::read_map(&mut array, "maps/arena2.map");
    // let star_time = Instant::now();
    // let _ = testing_functions::test_visualizer(&mut array);
    // let finish = star_time.elapsed().as_millis();
    // print!("{}ms", finish)


    testing_functions::test_astar_correctnes::<HashMap<Coords, SearchNode>>(&mut array, "hashmap");
    testing_functions::test_astar_correctnes::<robin_hood::RobinHood>(&mut array, "robin_hood");
    testing_functions::test_astar_correctnes::<judy::Judy>(&mut array, "judy");


    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
