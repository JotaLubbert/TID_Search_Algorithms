use std::time::{Duration, Instant};
use rand::{self, RngExt};
use crate::{CustomMap, a_star::{Coords, a_star}, distances_types::{self, octile_distance}, map_visualization, memory_counter::Medido, open_time::{Grabado, Repeticion}, open_list::{BinaryHeapOpen, OpenList, QuickHeapOpen, QuickHeapScalarOpen, RadixAltOpen, RadixAltRoundOpen, RadixHeapOpen, VebOpen}, read_files::{MapStats, decode_scen, read_folders, read_lines, read_map}, search_functions::search_valid_coords, write_files};
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
            //Medido envuelve la open para registrar el máximo de memoria que reservó en la búsqueda
            let astar_data = a_star::<Medido<OpenType>, _>(
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
                astar_data.open.peak_bytes(),
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

fn elapsed_to_min_secs(name: &str,elapsed: u64){
    let minutes = elapsed/60;
    let seconds = elapsed % 60;
    println!("Tiempo en ejecución de {name}: {minutes}min {seconds}s")
}

pub fn astar_diferent_structures(map:&mut CustomMap){
    let start_time = Instant::now();
    test_astar_correctnes::<BinaryHeapOpen>(map);
    let binary_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("BinaryHeap", binary_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixHeapOpen>(map);
    let radix_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("RadixHeap", radix_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixAltOpen>(map);
    let radix_alt_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("RadixAlt", radix_alt_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<RadixAltRoundOpen>(map); // amplifica el f * 2^(20), redondea y luego a u64
    let radix_alt_round_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("RadixAltRound", radix_alt_round_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<VebOpen>(map);
    let veb_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("vEB", veb_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<QuickHeapOpen>(map);
    let quick_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("QuickHeap", quick_elapsed);
    let start_time = Instant::now();
    test_astar_correctnes::<QuickHeapScalarOpen>(map);
    let quick_scalar_elapsed = start_time.elapsed().as_secs();
    elapsed_to_min_secs("QuickHeapScalar", quick_scalar_elapsed);
}

fn run_map_struct_save<OpenType>(map: &CustomMap, stats: MapStats, scen_files: &str)
where OpenType: OpenList,
{
    let star_time = Instant::now();
    //Medido envuelve la open para registrar el máximo de memoria que reservó en la búsqueda
    let astar_data = a_star::<Medido<OpenType>, _>(
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
        astar_data.open.peak_bytes(),
        finish
    );
}

pub fn test_structur_per_map(map:&mut CustomMap){
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
            run_map_struct_save::<BinaryHeapOpen>(map, stats, &scen_files);
            run_map_struct_save::<RadixHeapOpen>(map, stats, &scen_files);
            run_map_struct_save::<RadixAltOpen>(map, stats, &scen_files);
            run_map_struct_save::<RadixAltRoundOpen>(map, stats, &scen_files);
            run_map_struct_save::<VebOpen>(map, stats, &scen_files);
            run_map_struct_save::<QuickHeapOpen>(map, stats, &scen_files);
            run_map_struct_save::<QuickHeapScalarOpen>(map, stats, &scen_files); // el mismo QuickHeap sin SIMD

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
        println!("Finish map: {}", data_compare);
    }
}
//mapas donde se mide el tiempo de la open por resta (cargo run --release -- open-time).
//Una muestra de tipos distintos y tamaño moderado; con la lista vacía se miden todos los mapas
pub const OPEN_TIME_MAPS: &[&str] = &[
    "arena2", "den312d", "den401d", "hrt201n", "lak504d", "brc300d", "orz303d", "lgt601d",
];

//una búsqueda real grabando lo que devuelve la open, y la misma búsqueda repitiendo esa grabación
//con una open que no ordena nada. La diferencia de tiempo es lo que costó la open (ver open_time.rs)
fn run_open_time<OpenType>(map: &CustomMap, stats: MapStats, scen_files: &str)
where OpenType: OpenList,
{
    let star_time = Instant::now();
    let mut real = a_star::<Grabado<OpenType>, _>(
        stats.start,
        stats.goal,
        map,
        octile_distance
    ).unwrap();
    let real_ns = star_time.elapsed().as_nanos();
    real.open.preparar_repeticion();

    let star_time = Instant::now();
    let replay = a_star::<Repeticion, _>(
        stats.start,
        stats.goal,
        map,
        octile_distance
    ).unwrap();
    let replay_ns = star_time.elapsed().as_nanos();

    //la repetición tiene que haber hecho exactamente el mismo trabajo; si no, la resta no vale
    assert_eq!(real.expansions, replay.expansions, "la repetición no reprodujo la búsqueda de {}", OpenType::NAME);
    assert_eq!(real.path, replay.path, "la repetición no reprodujo el camino de {}", OpenType::NAME);
    write_files::create_open_time_file(
        scen_files,
        OpenType::NAME,
        stats.start,
        stats.goal,
        stats.distance,
        real.expansions,
        real_ns,
        replay_ns
    );
}

//como test_structur_per_map (las estructuras intercaladas en cada escenario), pero midiendo cuánto
//tiempo usa la open; solo en los mapas de only_maps, o en todos si la lista está vacía
pub fn test_open_time_per_map(map: &mut CustomMap, only_maps: &[&str]){
    let maps = read_folders("maps");
    let test_data = read_folders("test_data");
    for scen_files in test_data{
        let data_compare = match scen_files.strip_suffix(".scen"){
            Some(data)=> {data}
            None => {
                panic!("Error, probablemente estás leyendo el directorio equivocado");
            }
        };
        //data_compare es "arena2.map"; la lista usa el nombre sin ".map"
        let map_name = data_compare.strip_suffix(".map").unwrap_or(data_compare);
        if !only_maps.is_empty() && !only_maps.contains(&map_name) {
            continue;
        }
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
        read_map(map, &map_dir);
        //la primera línea del .scen es la versión del formato
        for line in read_lines(&data_in_dir).into_iter().skip(1){
            let stats = decode_scen(line);
            run_open_time::<BinaryHeapOpen>(map, stats, &scen_files);
            run_open_time::<RadixHeapOpen>(map, stats, &scen_files);
            run_open_time::<RadixAltOpen>(map, stats, &scen_files);
            run_open_time::<RadixAltRoundOpen>(map, stats, &scen_files);
            run_open_time::<VebOpen>(map, stats, &scen_files);
            run_open_time::<QuickHeapOpen>(map, stats, &scen_files);
            run_open_time::<QuickHeapScalarOpen>(map, stats, &scen_files);
        }
        println!("Tiempo de la open medido en {}", map_name);
    }
}
