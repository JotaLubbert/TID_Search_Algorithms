use std::time::Instant;
use rand::{self, RngExt};
use crate::{a_star::a_star, distances_types, map_visualization, search_functions::{search_all_valid_coords, search_valid_coords}};

pub fn test_astar(map:&mut[[bool; 512]; 512], test_atempts: usize)->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_valid_coords(map, 15);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::with_capacity(test_atempts);
    for _i in 0..test_atempts{
        let start = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let goal = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let star_time = Instant::now();
        let path = a_star(start, goal, map, distances_types::euclidean_distance);
        let finish = star_time.elapsed();
        let path = path.unwrap().1;
        end_test.push((finish.as_millis(), path));
    }
    return end_test;
}

pub fn test_all_valid_points(map:&mut[[bool; 512]; 512])->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_all_valid_coords(map);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::new();
    for (i, start) in test_coords.iter().enumerate(){
        for goal in &test_coords[i..]{
            let star_time = Instant::now();
            let path = a_star(*start, *goal, map, distances_types::euclidean_distance);
            let finish = star_time.elapsed();
            let path = path.unwrap().1;
            end_test.push((finish.as_millis(), path));
        }
    }
    return end_test;
}

pub fn test_visualizer(map:&mut[[bool; 512]; 512]){
    let start= (0, 0); let goal = (4, 1);
    let (_total_distance, path, open, close) = a_star(start, goal, map, distances_types::euclidean_distance).unwrap();
    map_visualization::visualize_final_state(map, &open, &close, start, goal, &path, 4, "generated_output/hi.png");
}