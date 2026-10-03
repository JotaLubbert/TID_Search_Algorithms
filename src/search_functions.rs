use rand::{self, RngExt};

use crate::CustomMap;

pub fn search_valid_coords(map: &CustomMap, k_points: u32)->Vec<(u32, u32)>{
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

pub fn search_all_valid_coords(map: &CustomMap)->Vec<(u32, u32)>{
    let mut valid_coords: Vec<(u32, u32)> = Vec::new();
    for j in 0..512{
        for i in 0..512{
            if map[j][i]{
                valid_coords.push((i as u32, j as u32));
            }
        }
    }
    return valid_coords;
}