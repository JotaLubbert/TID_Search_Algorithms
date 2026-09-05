use std::{time::Instant};
use rand::{self, RngExt};
use crate::{CustomMap, a_star::a_star, distances_types::{self, euclidean_distance}, map_visualization, read_files::{MapStats, decode_scen, read_folders, read_lines, read_map}, search_functions::{search_all_valid_coords, search_valid_coords}, write_files};

pub fn test_astar(map:&mut CustomMap, test_atempts: usize)->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_valid_coords(map, 15);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::with_capacity(test_atempts);
    for _i in 0..test_atempts{
        let start = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let goal = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let star_time = Instant::now();
        let astar_results = a_star(start, goal, map, distances_types::euclidean_distance);
        let finish = star_time.elapsed();
        end_test.push((finish.as_millis(), astar_results.unwrap().path));
    }
    return end_test;
}

pub fn test_all_valid_points(map:&mut CustomMap)->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_all_valid_coords(map);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::new();
    for (i, start) in test_coords.iter().enumerate(){
        for goal in &test_coords[i..]{
            let star_time = Instant::now();
            let astar_results = a_star(*start, *goal, map, distances_types::euclidean_distance).unwrap();
            let finish = star_time.elapsed();
            end_test.push((finish.as_millis(), astar_results.path));
        }
    }
    return end_test;
}

// pub fn test_visualizer(map:&mut CustomMap){
//     let start= (79, 144); let goal = (244, 78);
//     let (_total_distance, path, open, close) = a_star(start, goal, map, distances_types::euclidean_distance).unwrap();
//     map_visualization::visualize_final_state(map, &open, &close, start, goal, &path, 4, "generated_output/hi.png");
// }

pub fn test_astar_correctnes(map:&mut CustomMap){
    let maps = read_folders("maps");
    let test_data = read_folders("test_data");
    for scen_files in test_data{
        let data_compare = match scen_files.strip_suffix(".scen"){
            Some(data)=> {data}
            None => {
                panic!("Error, probablemente estás leyendo el directorio equivocado");
            }
        };
        let maptowork  = match maps.get(data_compare){
            Some(working_map)=> working_map,
            None => {
                println!("No se encontró mapa deseado.");
                continue;
            }
        };

        let map_dir = format!("maps/{}", maptowork);
        let data_in_dir = format!("test_data/{}", scen_files);
        let (height, width) = read_map(map, &map_dir);
        let data = read_lines(&data_in_dir);
        let mut first_line = true;
        for line in data{
            if first_line {
                first_line = false;
                continue;
            }
            let stats = decode_scen(line);
            let astar_data = a_star(
                stats.start,
                stats.goal,
                map,
                euclidean_distance
            ).unwrap();
            let file_name = scen_files.clone();
            write_files::create_stat_file(
                file_name,
                stats.start,
                stats.goal,
                stats.distance,
                astar_data.final_dis
            );
            /*
            let ouput_path = format!("generated_output/{}-{}_{}-{}_{}.png",
                &data_dir,
                stats.start.0,
                stats.start.1,
                stats.goal.0,
                stats.goal.1
            );
            map_visualization::visualize_final_state(
                map,
                width, height,
                &open, &close,
                stats.start, stats.goal,
                &path,
                4,
                &ouput_path
            );
            */
        }
    }
}