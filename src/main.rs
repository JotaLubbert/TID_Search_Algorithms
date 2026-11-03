mod a_star;
mod benchmark;
mod closed_set;
mod distances_types;
mod hasher;
mod judy;
mod map_visualization;
mod read_files;
mod robin_hood;
mod search_functions;
mod testing_functions;
mod timed;
mod write_files;

use std::collections::HashMap;
use crate::a_star::{Coords, SearchNode};
use crate::hasher::CoordBuild;
use crate::judy::Judy;
use crate::robin_hood::RobinHood;

type CustomMap = [[bool; 2048]; 2048];
fn main() {

    if cfg!(debug_assertions) {
        eprintln!("ERROR: compilado sin optimizaciones. Ejecuta con `cargo run --release`: en debug los tiempos no son comparables.");
        std::process::exit(1);
    }

    let mut array: Box<CustomMap> = vec![[false; 2048]; 2048].into_boxed_slice().try_into().unwrap();
    // read_files::read_map(&mut array, "maps/arena2.map");
    // let star_time = Instant::now();
    // let _ = testing_functions::test_visualizer(&mut array);
    // let finish = star_time.elapsed().as_millis();
    // print!("{}ms", finish)


    testing_functions::test_astar_correctnes::<HashMap<Coords, SearchNode>>(&mut array, "hashmap");
    testing_functions::test_astar_correctnes::<HashMap<Coords, SearchNode, CoordBuild>>(&mut array, "hashmap_hash_robin");
    testing_functions::test_astar_correctnes::<RobinHood>(&mut array, "robin_hood");
    testing_functions::test_astar_correctnes::<Judy>(&mut array, "judy");


    // benchmark::micro_directo();
    // benchmark::bench_directo(&mut array, "brc201d.map.scen", 5, 100);


    // testing_functions::test_one_map::<judy::Judy>(&mut array, "judy", "hrt000d.map.scen");

    // let mut total_time: u128 = 0;
    // for (time, path) in testing{
    //     println!("time: {} ms", time);
    //     println!("{:?}", path);
    //     total_time += time;
    // }
    // print!("{}", total_time);
}
