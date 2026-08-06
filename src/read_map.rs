use std::fs::read_to_string;
pub fn is_travesable(c: u8)->bool{
    return matches!(c, b'.' | b'G'); //El caracter 'S' tabién es traversable, pero es pantano, hay que ver que se hace
}

fn read_lines(path:&str)-> Vec<String>{
    read_to_string(path).unwrap() // panic on possible file-reading errors
        .lines()  // split the string into an iterator of string slices
        .map(String::from)  // make each slice into a string
        .collect()  // gather them together into a vector
}

pub fn read_map(arr:&mut[[bool; 512]; 512], file_directory: &str){
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

