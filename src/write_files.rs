use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;

use crate::{a_star::{AStarResults, Coords}, open_list::OpenList};

// columnas del .tsv, en el mismo orden que se escriben las filas.
// path va al final a proposito: es la unica de ancho variable, asi que
// dejarla ahi evita que el resto de las columnas se desalinee.
const HEADER: &str = "start_x\tstart_y\tgoal_x\tgoal_y\texpected_distance\tactual_distance\tpath_len\texpansions\tgenerated\topen_bytes\topen_used_bytes\topen_peak_bytes\tclose_bytes\texecution_time\tpath";

pub fn create_stat_file(
    mut file_name: String,
    start: Coords,
    goal: Coords,
    expected_distance: f64,
    results: &AStarResults<impl OpenList>,
    //máximo que reservó la open en la búsqueda (lo mide Medido, A* no lo conoce)
    open_peak_bytes: usize,
    execution_time: u128,
) {
    file_name.push_str(".tsv");

    //el camino completo, horizontal: "(x, y) (x, y) ...". Sin tabs ni saltos de
    //linea, que son los unicos caracteres que romperian el tsv.
    let mut path_str = String::with_capacity(results.path.len() * 12);
    for (i, coord) in results.path.iter().enumerate() {
        if i > 0 {
            path_str.push(' ');
        }
        path_str.push_str(&format!("({}, {})", coord.0, coord.1));
    }

    let text_to_write = format!(
        "{}\t{}\t{}\t{}\t{:.8}\t{:.8}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        start.0, start.1,
        goal.0, goal.1,
        expected_distance,
        results.final_dis,
        results.path.len(),
        results.expansions,
        results.generated,
        results.open_bytes,
        results.open_used_bytes,
        open_peak_bytes,
        results.close_bytes,
        execution_time,
        path_str
    );

    //se revisa la ruta completa: file_name puede incluir subcarpeta (ej. "binary-heap/..."),
    //y listar test_result/ solo ve las subcarpetas, no los archivos dentro de ellas.
    let rout_file = format!("test_result/{}", file_name);
    if !Path::new(&rout_file).exists() {
        let _ = fs::write(&rout_file, format!("{}\n", HEADER));
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(&rout_file)
        .unwrap();
    let _ = writeln!(file, "{}", text_to_write);
}

//tiempo de la open medido por resta (open_time.rs): un archivo por mapa en test_result/open-time,
//con una fila por escenario y estructura. Los tiempos van en nanosegundos: en los caminos cortos la
//diferencia entre la búsqueda real y su repetición es de pocos µs
const OPEN_TIME_DIR: &str = "test_result/open-time";
const OPEN_TIME_HEADER: &str = "structure\tstart_x\tstart_y\tgoal_x\tgoal_y\texpected_distance\texpansions\treal_ns\treplay_ns";

pub fn create_open_time_file(
    scen_file: &str,
    structure: &str,
    start: Coords,
    goal: Coords,
    expected_distance: f64,
    expansions: u64,
    real_ns: u128,
    replay_ns: u128,
) {
    //a diferencia de las carpetas de cada estructura, esta se crea sola si no existe
    let _ = fs::create_dir_all(OPEN_TIME_DIR);
    let rout_file = format!("{}/{}.tsv", OPEN_TIME_DIR, scen_file);
    if !Path::new(&rout_file).exists() {
        let _ = fs::write(&rout_file, format!("{}\n", OPEN_TIME_HEADER));
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(&rout_file)
        .unwrap();
    let _ = writeln!(
        file,
        "{}\t{}\t{}\t{}\t{}\t{:.8}\t{}\t{}\t{}",
        structure,
        start.0, start.1,
        goal.0, goal.1,
        expected_distance,
        expansions,
        real_ns,
        replay_ns
    );
}
