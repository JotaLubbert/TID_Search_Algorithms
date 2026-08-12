use rand::{self, RngExt};
use std::time::Instant;
use crate::{a_star::a_star, distances_types};

fn search_valid_coords(map: &[[bool; 512]; 512], k_points: u32)->Vec<(u32, u32)>{
    let mut succesfull_atemts:u32 = 0;
    let mut samples: Vec<(u32, u32)> = Vec::with_capacity(k_points as usize);
    while succesfull_atemts <= k_points {
        let posible_tuple = (rand::rng().random::<u32>() % 512, rand::rng().random::<u32>() % 512);
        if map[posible_tuple.1 as usize][posible_tuple.0 as usize]{
            samples.push(posible_tuple);
            succesfull_atemts += 1;
        }
    }
    return samples;
}

pub fn test_astar(map:&mut[[bool; 512]; 512], test_atempts: usize)->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_valid_coords(map, 15);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::with_capacity(test_atempts);
    for _i in 0..test_atempts{
        let start = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let goal = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let star_time = Instant::now();
        let path =a_star(start, goal, map, distances_types::euclidean_distance);
        let finish = star_time.elapsed();
        let path = path.unwrap().1;
        end_test.push((finish.as_millis(), path));
    }
    return end_test;
}
