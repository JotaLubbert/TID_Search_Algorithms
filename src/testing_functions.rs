use std::path::Path;
use std::time::{Duration, Instant};
use rand::{self, RngExt};
use crate::{CustomMap, a_star::{Coords, a_star}, distances_types::{self, octile_distance}, map_visualization, memory_counter::Medido, open_time::{Grabado, Repeticion}, open_list::{BinaryHeapOpen, OpenList, QuickHeapOpen, QuickHeapScalarOpen, RadixAltRoundOpen, RadixHeapOpen, VebOpen}, read_files::{MapStats, decode_scen, read_folders, read_lines, read_map}, search_functions::search_valid_coords, write_files};
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
    recorrer_escenarios(map, |map, stats, scen_file| run_map_struct_save::<OpenType>(map, stats, scen_file));
}

//cada colección de mapas es una subcarpeta con el mismo nombre en maps/ y en test_data/
//(ej. maps/Mazes/x.map y test_data/Mazes/x.map.scen)
const MAPS_DIR: &str = "maps";
const SCEN_DIR: &str = "test_data";

//de cada .scen se corre 1 escenario de cada `paso`. Mazes trae 627.000 escenarios y cada búsqueda
//tarda ~8 veces lo de DragonAges: completos sumarían ~130 h a la corrida. Con 1 de cada 40 quedan
//15.675, repartidos en los 60 mapas y en todo el rango de largos. Las colecciones que no están acá van completas
const MUESTREO: &[(&str, usize)] = &[("Mazes", 40)];

fn paso_de(coleccion: &str) -> usize {
    MUESTREO.iter().find(|(nombre, _)| *nombre == coleccion).map_or(1, |&(_, paso)| paso)
}

//carga cada mapa con escenarios y le pasa a `correr` sus escenarios (ya muestreados), uno por uno,
//junto con "colección/archivo.scen", que es donde van sus resultados dentro de cada estructura
fn recorrer_escenarios(map: &mut CustomMap, mut correr: impl FnMut(&CustomMap, MapStats, &str)) {
    let mut colecciones: Vec<String> = read_folders(SCEN_DIR).into_iter()
        .filter(|nombre| Path::new(SCEN_DIR).join(nombre).is_dir())
        .collect();
    colecciones.sort();
    for coleccion in colecciones {
        let maps = read_folders(&format!("{MAPS_DIR}/{coleccion}"));
        let paso = paso_de(&coleccion);
        for scen_file in read_folders(&format!("{SCEN_DIR}/{coleccion}")) {
            let Some(map_name) = scen_file.strip_suffix(".scen") else {
                panic!("{SCEN_DIR}/{coleccion}/{scen_file} no es un .scen, probablemente estás leyendo el directorio equivocado");
            };
            if !maps.contains(map_name) {
                println!("No se encontró el mapa {MAPS_DIR}/{coleccion}/{map_name}.");
                continue;
            }
            *map = [[false; 2048]; 2048];
            read_map(map, &format!("{MAPS_DIR}/{coleccion}/{map_name}"));
            //la primera línea del .scen es la versión del formato
            let escenarios = read_lines(&format!("{SCEN_DIR}/{coleccion}/{scen_file}"));
            let destino = format!("{coleccion}/{scen_file}");
            for line in escenarios.into_iter().skip(1).step_by(paso) {
                correr(map, decode_scen(line), &destino);
            }
            println!("Finish map: {coleccion}/{map_name}");
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
    test_astar_correctnes::<RadixAltRoundOpen>(map); // la clave común open_key(f): f * 2^18 redondeada
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
    //Medido registra el máximo de memoria que reservó la open; Grabado anota qué nodos salieron de
    //ella, para la repetición. Grabado va afuera para que su anotación no cuente como memoria de la open
    let mut astar_data = a_star::<Grabado<Medido<OpenType>>, _>(
        stats.start,
        stats.goal,
        map,
        octile_distance
    ).unwrap();
    let search_time = star_time.elapsed();
    let finish = search_time.as_micros();

    //la misma búsqueda con una open que solo repite lo grabado, sin ordenar nada:
    //lo que tarda de menos es lo que usó la open (ver open_time.rs)
    astar_data.open.preparar_repeticion();
    let star_time = Instant::now();
    let replay = a_star::<Repeticion, _>(
        stats.start,
        stats.goal,
        map,
        octile_distance
    ).unwrap();
    let replay_time = star_time.elapsed();
    //la repetición tiene que haber hecho exactamente el mismo trabajo; si no, la resta no vale
    assert_eq!(astar_data.expansions, replay.expansions, "la repetición no reprodujo la búsqueda de {}", OpenType::NAME);
    let open_time_ns = search_time.as_nanos() as i64 - replay_time.as_nanos() as i64;

    let file_name = format!("{}/{}", OpenType::NAME, scen_files);
    write_files::create_stat_file(
        file_name,
        stats.start,
        stats.goal,
        stats.distance,
        &astar_data,
        astar_data.open.inner().peak_bytes(),
        finish,
        open_time_ns
    );
}

pub fn test_structur_per_map(map:&mut CustomMap){
    recorrer_escenarios(map, |map, stats, scen_file| {
        run_map_struct_save::<BinaryHeapOpen>(map, stats, scen_file);
        run_map_struct_save::<RadixHeapOpen>(map, stats, scen_file);
        run_map_struct_save::<RadixAltRoundOpen>(map, stats, scen_file);
        run_map_struct_save::<VebOpen>(map, stats, scen_file);
        run_map_struct_save::<QuickHeapOpen>(map, stats, scen_file);
        run_map_struct_save::<QuickHeapScalarOpen>(map, stats, scen_file); // el mismo QuickHeap sin SIMD
    });
    //para dibujar cada búsqueda (hay que traer astar_data y el ancho/alto que devuelve read_map):
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
