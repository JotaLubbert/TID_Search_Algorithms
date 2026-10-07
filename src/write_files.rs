use std::fs::{self, OpenOptions};
use std::io::{Write};

use crate::{a_star::{AStarResults, Coords}, read_files::read_folders};

// columnas del .tsv, en el mismo orden que se escriben las filas.
// path va al final a proposito: es la unica de ancho variable, asi que
// dejarla ahi evita que el resto de las columnas se desalinee.
const HEADER: &str = "start_x\tstart_y\tgoal_x\tgoal_y\texpected_distance\tactual_distance\tpath_len\texpansions\tgenerated\topen_nodes\tclose_nodes\topen_node_bytes\tclose_node_bytes\topen_bytes\tclose_bytes\texecution_time\tpath";

pub fn create_stat_file(
    label: &str,
    mut file_name: String,
    start: Coords,
    goal: Coords,
    expected_distance: f64,
    results: &AStarResults,
    execution_time: u128,
) {
    file_name.push_str(".tsv");
    let dir = format!("test_result/{}", label);
    fs::create_dir_all(&dir).unwrap();
    let files = read_folders(&dir);

    let mut path_str = String::with_capacity(results.path.len() * 12);
    for (i, coord) in results.path.iter().enumerate() {
        if i > 0 {
            path_str.push(' ');
        }
        path_str.push_str(&format!("({}, {})", coord.0, coord.1));
    }

   let text_to_write = format!(
        "{}\t{}\t{}\t{}\t{:.8}\t{:.8}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        start.0, start.1,
        goal.0, goal.1,
        expected_distance,
        results.final_dis,
        results.path.len(),
        results.expansions,
        results.generated,
        results.open_nodes,
        results.close_nodes,
        results.open_node_bytes,
        results.close_node_bytes,
        results.open_bytes,
        results.close_bytes,
        execution_time,
        path_str
    );

    let rout_file = format!("{}/{}", dir, file_name);
    if !files.contains(&file_name) {
        fs::write(&rout_file, format!("{}\n", HEADER)).unwrap();
    }
    let mut file = OpenOptions::new().append(true).open(&rout_file).unwrap();
    let _ = writeln!(file, "{}", text_to_write);
}
