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
use crate::testing_functions::test_astar_correctnes;

type CustomMap = [[bool; 2048]; 2048];

fn main() {
    // Sin --release los tiempos no son comparables.
    if cfg!(debug_assertions) {
        eprintln!("ERROR: compilado sin optimizaciones. Ejecuta con `cargo run --release`: en debug los tiempos no son comparables.");
        std::process::exit(1);
    }

    let mut array: Box<CustomMap> = vec![[false; 2048]; 2048].into_boxed_slice().try_into().unwrap();

    // RECORDAR CAMBIAR ITERACIÓN AL CORRER

    // test_astar_correctnes::<HashMap<Coords, SearchNode>>(&mut array, "hashmap", "tradicionales", 2, false);
    // test_astar_correctnes::<HashMap<Coords, SearchNode, CoordBuild>>(&mut array, "hashmap_hash_robin", "tradicionales", 2, false);
    test_astar_correctnes::<RobinHood>(&mut array, "robin_hood", "tradicionales", 2, false);
    test_astar_correctnes::<Judy>(&mut array, "judy", "tradicionales", 2, false);

    // test_astar_correctnes::<HashMap<Coords, SearchNode>>(&mut array, "hashmap", "laberintos", 1, true);
    // test_astar_correctnes::<HashMap<Coords, SearchNode, CoordBuild>>(&mut array, "hashmap_hash_robin", "laberintos", 1, true);
    // test_astar_correctnes::<RobinHood>(&mut array, "robin_hood", "laberintos", 1, true);
    // test_astar_correctnes::<Judy>(&mut array, "judy", "laberintos", 1, true);


    // benchmark::micro_directo();
    // benchmark::bench_directo(&mut array, "tradicionales", "brc201d.map.scen", 5, 100);
}