mod read_map;
mod a_star;
mod random_functions;
//mod a_estrella;
mod distances_types;
fn main(){
    let mut array: [[bool; 512]; 512] = [[false; 512]; 512];
    read_map::read_map(&mut array, "test_maps/arena2.map");
    //print!("{:?}", array[0]);
    //let res = a_star::a_star((0, 0), (5,5), &mut array, distances_types::euclidean_distance);
    let test_results = random_functions::test_astar(&mut array, 15);
    for (time, path) in test_results{
        println!("Time: {}ms", time);
        println!("path: ");
        println!("{:?}", path);
        
    //let res = a_estrella::astar((107, 76), (171, 140), &array, distances_types::euclidean_distance);
    //match res {
    //    Some((path, cost)) => {
    //        println!("costo de a_estrella: {}", cost);
    //        println!("path de a_estrella: {:?}", path);
    //    }
    //    None => println!("a_estrella: sin camino")
    }
}
