mod a_star;
mod open_list;
mod read_files;
mod search_functions;
mod testing_functions;
mod write_files;
mod distances_types;
mod map_visualization;
mod veb;
mod radix_alt;

type CustomMap = [[bool; 2048]; 2048];
fn main() {
    let mut array: CustomMap = [[false; 2048]; 2048];
    // read_files::read_map(&mut array, "maps/arena2.map");
    // let star_time = Instant::now();
    // let _ = testing_functions::test_visualizer(&mut array);
    // let finish = star_time.elapsed().as_millis();
    // print!("{}ms", finish)


    testing_functions::astar_diferent_structures(&mut array);


    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
