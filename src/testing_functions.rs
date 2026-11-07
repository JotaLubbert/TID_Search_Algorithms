use std::time::Instant;
use std::collections::HashMap;
use rand::{self, RngExt};
use crate::timed::{self, Timed};
use std::collections::HashSet;
use std::io::Write;
use crate::{
    CustomMap,
    a_star::{a_star, Coords, SearchNode},
    closed_set::ClosedSet,
    distances_types::{self, euclidean_distance},
    read_files::{decode_scen, read_folders, read_lines, read_map},
    search_functions::{search_all_valid_coords, search_valid_coords},
    write_files,
};

pub const GRUPOS: [(&str, &str, &str); 2] = [
    ("tradicionales", "test_data", "maps"),
    ("laberintos", "maze_scen", "maps"), // si separas los .map de laberintos, cambia "maps" por esa carpeta
];

pub fn carpetas_grupo(grupo: &str) -> (&'static str, &'static str) {
    for (g, scen_dir, maps_dir) in GRUPOS {
        if g == grupo {
            return (scen_dir, maps_dir);
        }
    }
    panic!("grupo desconocido: {}", grupo);
}

pub fn test_astar(map: &mut CustomMap, test_atempts: usize) -> Vec<(u128, Vec<(u32, u32)>)> {
    let test_coords = search_valid_coords(map, 15);
    let mut end_test: Vec<(u128, Vec<(u32, u32)>)> = Vec::with_capacity(test_atempts);
    for _i in 0..test_atempts {
        let start = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let goal = test_coords[(rand::rng().random::<u32>() % 15) as usize];
        let star_time = Instant::now();
        let astar_results = a_star::<HashMap<Coords, SearchNode>, _>(start, goal, map, distances_types::euclidean_distance);
        let finish = star_time.elapsed();
        end_test.push((finish.as_millis(), astar_results.unwrap().path));
    }
    return end_test;
}

pub fn test_all_valid_points(map: &mut CustomMap) -> Vec<(u128, Vec<(u32, u32)>)> {
    let test_coords = search_all_valid_coords(map);
    let mut end_test: Vec<(u128, Vec<(u32, u32)>)> = Vec::new();
    for (i, start) in test_coords.iter().enumerate() {
        for goal in &test_coords[i..] {
            let star_time = Instant::now();
            let astar_results = a_star::<HashMap<Coords, SearchNode>, _>(*start, *goal, map, distances_types::euclidean_distance).unwrap();
            let finish = star_time.elapsed();
            end_test.push((finish.as_millis(), astar_results.path));
        }
    }
    return end_test;
}

// pub fn test_visualizer(map: &mut CustomMap) {
//     let start = (79, 144); let goal = (244, 78);
//     let (_total_distance, path, open, close) = a_star::<HashMap<Coords, SearchNode>, _>(start, goal, map, distances_types::euclidean_distance).unwrap();
//     map_visualization::visualize_final_state(map, &open, &close, start, goal, &path, 4, "generated_output/hi.png");
// }

// `C` es la estructura del closed y `label` el nombre de la subcarpeta en test_result/
// Marcas de "mapa completo": test_result/<estructura>/completos_<carpeta>.txt
// Permiten continuar una corrida cortada sin repetir los mapas ya terminados.
fn leer_completos(ruta: &str) -> HashSet<String> {
    std::fs::read_to_string(ruta)
        .map(|s| s.lines().map(String::from).collect::<HashSet<String>>())
        .unwrap_or_default()
}

fn marcar_completo(ruta: &str, scen: &str) {
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(ruta).unwrap();
    let _ = writeln!(f, "{}", scen);
}

pub fn test_astar_correctnes<C: ClosedSet>(map: &mut CustomMap, label: &str, grupo: &str, repeticion: u32, reanudar: bool) {
    let (scen_dir, maps_dir) = carpetas_grupo(grupo);
    if !std::path::Path::new(scen_dir).is_dir() {
        println!("No existe la carpeta {}: no se puede correr el grupo {}", scen_dir, grupo);
        return;
    }
    let salida = if repeticion <= 1 { grupo.to_string() } else { format!("{}_{}", grupo, repeticion) };
    let destino = format!("{}/{}", label, salida);
    let completos = format!("test_result/{}/completos_{}.txt", label, salida);
    let _ = std::fs::create_dir_all(format!("test_result/{}", label));

    if !reanudar {
        let _ = std::fs::remove_dir_all(format!("test_result/{}", destino));
        let _ = std::fs::remove_file(&completos);
    }
    let hechos = leer_completos(&completos);

    let sobrecosto = timed::calibrar();
    println!("[{} · {}] sobrecosto del cronómetro: {:.1} ns por medición", label, salida, sobrecosto);

    let maps = read_folders(maps_dir);
    let mut escenarios: Vec<String> = read_folders(scen_dir).into_iter().collect();
    escenarios.sort();
    let total = escenarios.len();

    for (i, scen_files) in escenarios.iter().enumerate() {
        let data_compare = match scen_files.strip_suffix(".scen") {
            Some(data) => data,
            None => {
                println!("Se omite {} (no es un .scen)", scen_files);
                continue;
            }
        };
        let maptowork = match maps.get(data_compare) {
            Some(working_map) => working_map,
            None => {
                println!("No se encontró el mapa de {}.", scen_files);
                continue;
            }
        };
        if hechos.contains(scen_files) {
            println!("[{} · {}] ({}/{}) {} ya completo, se omite", label, salida, i + 1, total, scen_files);
            continue;
        }
        println!("[{} · {}] ({}/{}) {}", label, salida, i + 1, total, scen_files);

        // se descartan las filas parciales de una corrida cortada
        let _ = std::fs::remove_file(format!("test_result/{}/{}.tsv", destino, scen_files));

        let map_dir = format!("{}/{}", maps_dir, maptowork);
        let data_in_dir = format!("{}/{}", scen_dir, scen_files);
        let (_height, _width) = read_map(map, &map_dir);
        let mut first_line = true;
        for line in read_lines(&data_in_dir) {
            if first_line {
                first_line = false;
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            let stats = decode_scen(line);

            // corrida limpia: mide execution_time
            let star_time = Instant::now();
            let astar_data = match a_star::<C, _>(stats.start, stats.goal, map, euclidean_distance) {
                Some(d) => d,
                None => {
                    println!("Sin camino en {}: {:?} -> {:?}", scen_files, stats.start, stats.goal);
                    continue;
                }
            };
            let finish = star_time.elapsed().as_micros();

            // segunda corrida: cronómetro en cada get e insert del Closed
            timed::reset_stats();
            let _ = a_star::<Timed<C>, _>(stats.start, stats.goal, map, euclidean_distance);
            let closed = timed::take_stats();

            write_files::create_stat_file(
                &destino,
                scen_files.clone(),
                stats.start,
                stats.goal,
                stats.distance,
                &astar_data,
                finish,
                closed,
            );
        }
        marcar_completo(&completos, scen_files);
    }
}

pub fn test_one_map<C: ClosedSet>(map: &mut CustomMap, label: &str, grupo: &str, scen_name: &str) {
    timed::calibrar();
    let (scen_dir, maps_dir) = carpetas_grupo(grupo);
    let map_name = scen_name.strip_suffix(".scen").expect("el nombre debe terminar en .scen");
    let map_dir = format!("{}/{}", maps_dir, map_name);
    let data_in_dir = format!("{}/{}", scen_dir, scen_name);
    let destino = format!("{}/{}", label, grupo);

    // borra el resultado anterior para no duplicar filas
    let _ = std::fs::remove_file(format!("test_result/{}/{}.tsv", destino, scen_name));

    let (_height, _width) = read_map(map, &map_dir);
    for (i, line) in read_lines(&data_in_dir).into_iter().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let stats = decode_scen(line);
        let star_time = Instant::now();
        let astar_data = match a_star::<C, _>(stats.start, stats.goal, map, euclidean_distance) {
            Some(d) => d,
            None => {
                println!("Sin camino en {}: {:?} -> {:?}", scen_name, stats.start, stats.goal);
                continue;
            }
        };
        let finish = star_time.elapsed().as_micros();

        timed::reset_stats();
        let _ = a_star::<Timed<C>, _>(stats.start, stats.goal, map, euclidean_distance);
        let closed = timed::take_stats();

        write_files::create_stat_file(
            &destino,
            scen_name.to_string(),
            stats.start,
            stats.goal,
            stats.distance,
            &astar_data,
            finish,
            closed,
        );
    }
}