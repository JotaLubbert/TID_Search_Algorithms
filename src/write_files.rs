use std::fs::{self, OpenOptions};
use std::io::{Write};

use crate::{a_star::Coords, read_files::read_folders};


pub fn create_stat_file(mut file_name: String, start: Coords, goal: Coords, expected_distance: f64, actual_distance: f64) {
    file_name.push_str(".cmp");
    let files = read_folders("test_result");

    // anchos de columna — ajusta según el largo máximo esperado de tus datos
    const COL_COORDS: usize = 20;
    const COL_DIST: usize = 20;

    let start_str = format!("{:?}", start);
    let goal_str = format!("{:?}", goal);

    let text_to_write = format!(
        "{:^col_c$}{:^col_c$}{:^col_d$}{:^col_d$}",
        start_str, goal_str,
        format!("{:.8}", expected_distance),
        format!("{:.8}", actual_distance),
        col_c = COL_COORDS,
        col_d = COL_DIST
    );

    let header = format!(
        "{:^col_c$}{:^col_c$}{:^col_d$}{:^col_d$}",
        "Start_coords", "Goal", "Expected_distance", "Actual_distance",
        col_c = COL_COORDS,
        col_d = COL_DIST
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
            let _ = fs::write(&rout_file, format!("{}\n", header));
            let mut file = OpenOptions::new()
                .append(true)
                .open(&rout_file)
                .unwrap();
            let _ = writeln!(file, "{}", text_to_write);
        }
    }
}