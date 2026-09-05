mod a_star;
mod read_files;
use std::time::Instant;

use crate::read_files::{decode_scen, read_lines};
mod search_functions;
mod testing_functions;
//mod a_estrella;
mod distances_types;
mod map_visualization;
fn main() {
    // let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    // read_files::read_map(&mut array, "maps/arena2.map");
    // let star_time = Instant::now();
    // let _ = testing_functions::test_visualizer(&mut array);
    // let finish = star_time.elapsed().as_millis();
    // print!("{}ms", finish)


    let text = read_lines("test_data/arena.map.scen");
    for (i, val) in text.iter().enumerate(){
        if i == 0 {continue;}
        let val = val.clone();
        decode_scen(val);
    }


    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
