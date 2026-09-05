use std::{collections::HashSet, fs::read_to_string, io::SeekFrom::Start, os::linux::raw::stat};
use rand::distr::Map;
use std::fs;

use crate::CustomMap;
#[derive(Clone, Copy, Debug)]
pub struct MapStats{
    pub start: (u32, u32),
    pub goal: (u32, u32),
    pub distance: f64,
}

pub fn is_travesable(c: u8)->bool{
    return matches!(c, b'.' | b'G' | b'S');
}

pub fn read_lines(path:&str)-> Vec<String>{
    read_to_string(path).unwrap() // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}

pub fn read_map(arr:&mut CustomMap, file_directory: &str){
    let lines: Vec<String> = read_lines(file_directory);
    for (line, text) in lines.iter().enumerate() {
        if line < 4{
            continue;
        }
        for (index, characters) in text.chars().enumerate(){
            let ascii =  characters as u8;
            arr[line-4][index] = is_travesable(ascii);
        }
    }
}

pub fn decode_scen(line: String)->MapStats{
    let fields: Vec<&str> = line.split('\t').map(|f| f.trim()).collect();

    let start_x = fields[4].parse::<u32>().unwrap();
    let start_y = fields[5].parse::<u32>().unwrap();
    let goal_x = fields[6].parse::<u32>().unwrap();
    let goal_y= fields[7].parse::<u32>().unwrap();
    let distance: f64 = fields[8].parse::<f64>().unwrap();

    return MapStats{
        start: (start_x, start_y),
        goal: (goal_x, goal_y),
        distance: distance
    };
}

pub fn read_folders(folder: &str)->HashSet<String>{
    let paths = fs::read_dir(folder).unwrap();
    let mut files_found: HashSet<String> = HashSet::new();
    for file in paths{
        let name = file.unwrap();
        let name = name.file_name().to_str().unwrap().to_string();
        files_found.insert(name);
    }
    return files_found;
}