use std::{time::Instant};
use rand::{self, RngExt};
use crate::{CustomMap, a_star::a_star, distances_types::{self, euclidean_distance, octile_distance}, map_visualization, open_list::{self, BinaryHeapOpen, OpenList, RadixAltOpen, RadixAltRoundOpen, RadixHeapOpen, VebOpen}, read_files::{MapStats, decode_scen, read_folders, read_lines, read_map}, search_functions::{search_all_valid_coords, search_valid_coords}, write_files};
#[allow(dead_code)]
pub fn test_astar(map:&mut CustomMap, test_atempts: usize)->Vec<(u128, Vec<(u32, u32)>)>{
    let test_coords = search_valid_coords(map, 15);
    let mut end_test:Vec<(u128, Vec<(u32, u32)>)> = Vec::with_capacity(test_atempts);
    for _i in 0..test_atempts{
        let start = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let goal = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let star_time = Instant::now();
        let astar_results = a_star::<BinaryHeapOpen, _>(start, goal, map, distances_types::euclidean_distance);
        let finish = star_time.elapsed();
        end_test.push((finish.as_millis(), astar_results.unwrap().path));
    }
    return end_test;
}

pub fn test_astar_correctnes<OpenType>(map:&mut CustomMap)
where OpenType: OpenList,
{
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
        *map = [[false; 2048]; 2048];
        let (height, width) = read_map(map, &map_dir);
        let data = read_lines(&data_in_dir);
        let mut first_line = true;
        for line in data{
            if first_line {
                first_line = false;
                continue;
            }
            let stats = decode_scen(line);
            let star_time = Instant::now();
            let astar_data = a_star::<OpenType, _>(
                stats.start,
                stats.goal,
                map,
                octile_distance
            ).unwrap();
            let finish = star_time.elapsed().as_micros();
            let file_name = format!("{}/{}", OpenType::NAME, scen_files);
            write_files::create_stat_file(
                file_name,
                stats.start,
                stats.goal,
                stats.distance,
                &astar_data,
                finish
            );
            /*
            let ouput_path = format!("generated_output/{}-{}_{}-{}_{}.png",
                &scen_files,
                stats.start.0,
                stats.start.1,
                stats.goal.0,
                stats.goal.1
            );
            map_visualization::visualize_final_state(
                map,
                width, height,
                &astar_data.open, &astar_data.close,
                stats.start, stats.goal,
                &astar_data.path,
                4,
                &ouput_path
            );
            */
        }
    }
}

pub fn astar_diferent_structures(map:&mut CustomMap){
    let start_time = Instant::now();
    test_astar_correctnes::<BinaryHeapOpen>(map);
    let binary_elapsed = start_time.elapsed().as_secs();
    let minutes = binary_elapsed/60;
    let seconds = binary_elapsed % 60;
    println!("Tiempo en ejecución de BinaryHeap: {}min {}sec", minutes, seconds);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixHeapOpen>(map);
    let radix_elapsed = start_time.elapsed().as_secs();
    let minutes = radix_elapsed/60;
    let seconds = radix_elapsed % 60;
    println!("Tiempo en ejecución de RadixHeap: {}min {}sec", minutes, seconds);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixAltOpen>(map);
    let radix_alt_elapsed = start_time.elapsed().as_secs();
    let minutes = radix_alt_elapsed/60;
    let seconds = radix_alt_elapsed % 60;
    println!("Tiempo en ejecución de RadixAlt: {}min {}sec", minutes, seconds);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixAltRoundOpen>(map); // amplifica el f * 2^(20), redondea y luego a u64
    let radix_alt_round_elapsed = start_time.elapsed().as_secs();
    let minutes = radix_alt_round_elapsed/60;
    let seconds = radix_alt_round_elapsed % 60;
    println!("Tiempo en ejecución de RadixAltRound: {}min {}sec", minutes, seconds);
    let start_time = Instant::now();
    test_astar_correctnes::<VebOpen>(map);
    let veb_elapsed = start_time.elapsed().as_secs();
    let minutes = veb_elapsed/60;
    let seconds = veb_elapsed % 60;
    println!("Tiempo en ejecución de vEB: {}min {}sec", minutes, seconds);
}
