use std::fs::{self, OpenOptions};
use std::io::{Write};

use crate::{a_star::{AStarResults, Coords}, read_files::read_folders};

// columnas del .tsv, en el mismo orden que se escriben las filas
const HEADER: &str = "start_x\tstart_y\tgoal_x\tgoal_y\texpected_distance\tactual_distance\tpath_len\texpansions\tgenerated\topen_bytes\tclose_bytes\texecution_time";

pub fn create_stat_file(
    mut file_name: String,
    start: Coords,
    goal: Coords,
    expected_distance: f64,
    results: &AStarResults,
    execution_time: u128,
) {
    file_name.push_str(".tsv");
    let files = read_folders("test_result");

    let text_to_write = format!(
        "{}\t{}\t{}\t{}\t{:.8}\t{:.8}\t{}\t{}\t{}\t{}\t{}\t{}",
        start.0, start.1,
        goal.0, goal.1,
        expected_distance,
        results.final_dis,
        results.path.len(),
        results.expansions,
        results.generated,
        results.open_bytes,
        results.close_bytes,
        execution_time
    );

    match files.get(&file_name) {
        Some(name_file) => {
            let rout_file = format!("test_result/{}", name_file);
            let mut file = OpenOptions::new()
                .append(true)
                .open(rout_file)
                .unwrap();
            let _ = writeln!(file, "{}", text_to_write);
        }
        None => {
            let rout_file = format!("test_result/{}", file_name);
            let _ = fs::write(&rout_file, format!("{}\n", HEADER));
            let mut file = OpenOptions::new()
                .append(true)
                .open(&rout_file)
                .unwrap();
            let _ = writeln!(file, "{}", text_to_write);
        }
    }
}
