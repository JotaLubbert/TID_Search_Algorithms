use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use rand::{self, RngExt};
use crate::{CustomMap, a_star::{Coords, a_star}, distances_types::{self, octile_distance}, map_visualization, memory_counter::{Medido, SinTamanos}, open_time::{Cronometrado, calibrar_reloj}, open_list::{BinaryHeapOpen, OpenList, QuickHeapOpen, QuickHeapScalarOpen, RadixAltRoundOpen, RadixHeapOpen, VebOpen}, read_files::{MapStats, decode_scen, read_folders, read_lines, read_map}, search_functions::search_valid_coords, write_files};
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

//cada colección de mapas es una subcarpeta con el mismo nombre en maps/ y en test_data/
//(ej. maps/Starcraft/x.map y test_data/Starcraft/x.map.scen)
const MAPS_DIR: &str = "maps";
const SCEN_DIR: &str = "test_data";

//colecciones que están en test_data/ pero no se corren: Mazes no va en el informe
const EXCLUIDAS: &[&str] = &["Mazes"];

//de cada .scen se corre 1 escenario de cada `paso`; las colecciones que no están acá van completas.
//Hoy van todas completas (el informe exige correr todos los escenarios)
const MUESTREO: &[(&str, usize)] = &[];

fn paso_de(coleccion: &str) -> usize {
    MUESTREO.iter().find(|(nombre, _)| *nombre == coleccion).map_or(1, |&(_, paso)| paso)
}

//cuántas veces se corre todo y dónde queda cada cosa (ver write_files.rs):
//- test_result/busquedas: lo que no cambia entre corridas (camino, expansiones, memoria). Lo escribe
//  la corrida 1
//- test_result/corrida_k: los tiempos de la corrida k. La primera es completa (tiempo total y tiempo
//  de la open con cronómetro); las demás repiten solo la búsqueda limpia, para promediar el tiempo
//  total entre corridas
const CORRIDAS: usize = 3;
const RESULTADOS: &str = "test_result";
const BUSQUEDAS: &str = "test_result/busquedas";

//las estructuras que se comparan, en el orden en que corren en cada escenario
const ESTRUCTURAS: [&str; 6] = [
    <BinaryHeapOpen as OpenList>::NAME,
    <RadixHeapOpen as OpenList>::NAME,
    <RadixAltRoundOpen as OpenList>::NAME,
    <VebOpen as OpenList>::NAME,
    <QuickHeapOpen as OpenList>::NAME,
    <QuickHeapScalarOpen as OpenList>::NAME,
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Modo {
    //búsqueda limpia (tiempo total y memoria) más una pasada con cronómetro (tiempo de la open)
    Completa,
    //solo la búsqueda limpia
    SoloBusqueda,
}

//true si a este mapa le falta algo en alguna estructura, en alguna de las `carpetas` que escribe la
//corrida. Un mapa a medias (la corrida se cortó mientras se escribía) se borra entero, en todas las
//estructuras y esas carpetas, para correrlo de nuevo: así, al relanzar, la corrida sigue donde iba
fn mapa_pendiente(carpetas: &[String], destino: &str, escenarios: usize) -> bool {
    let rutas: Vec<String> = carpetas.iter()
        .flat_map(|carpeta| ESTRUCTURAS.iter().map(move |e| format!("{carpeta}/{e}/{destino}.tsv")))
        .collect();
    //el encabezado más una fila por escenario, cada una terminada en salto de línea
    let completo = |ruta: &String| {
        fs::read(ruta).is_ok_and(|bytes| bytes.iter().filter(|&&c| c == b'\n').count() == escenarios + 1)
    };
    if rutas.iter().all(completo) {
        return false;
    }
    for ruta in &rutas {
        let _ = fs::remove_file(ruta);
    }
    true
}

//carga cada mapa con escenarios y le pasa a `correr` sus escenarios (ya muestreados), uno por uno,
//junto con "colección/archivo.scen", que es donde van sus resultados dentro de cada estructura, y
//el número del escenario en el .scen (desde 0, sin la línea de la versión).
//Se salta los mapas que ya están completos en `carpetas` y anota el avance en salida/progreso.txt
fn recorrer_escenarios(map: &mut CustomMap, salida: &str, carpetas: &[String], mut correr: impl FnMut(&CustomMap, MapStats, &str, usize)) {
    let inicio = Instant::now();
    fs::create_dir_all(salida).expect("no se pudo crear la carpeta de la corrida");
    let mut progreso = fs::OpenOptions::new().create(true).append(true)
        .open(format!("{salida}/progreso.txt")).expect("no se pudo abrir progreso.txt");
    let mut anotar = |texto: String| {
        println!("{texto}");
        let _ = std::io::Write::write_all(&mut progreso, format!("{texto}\n").as_bytes());
    };

    let mut colecciones: Vec<String> = read_folders(SCEN_DIR).into_iter()
        .filter(|nombre| Path::new(SCEN_DIR).join(nombre).is_dir() && !EXCLUIDAS.contains(&nombre.as_str()))
        .collect();
    colecciones.sort();
    for coleccion in colecciones {
        let maps = read_folders(&format!("{MAPS_DIR}/{coleccion}"));
        let paso = paso_de(&coleccion);
        //en orden alfabético, para que todas las corridas recorran los mapas igual
        let mut scen_files: Vec<String> = read_folders(&format!("{SCEN_DIR}/{coleccion}")).into_iter().collect();
        scen_files.sort();
        for scen_file in scen_files {
            let Some(map_name) = scen_file.strip_suffix(".scen") else {
                panic!("{SCEN_DIR}/{coleccion}/{scen_file} no es un .scen, probablemente estás leyendo el directorio equivocado");
            };
            if !maps.contains(map_name) {
                anotar(format!("No se encontró el mapa {MAPS_DIR}/{coleccion}/{map_name}."));
                continue;
            }
            //la primera línea del .scen es la versión del formato
            let escenarios = read_lines(&format!("{SCEN_DIR}/{coleccion}/{scen_file}"));
            let destino = format!("{coleccion}/{scen_file}");
            if !mapa_pendiente(carpetas, &destino, escenarios.len().saturating_sub(1).div_ceil(paso)) {
                anotar(format!("Ya estaba: {coleccion}/{map_name}"));
                continue;
            }
            *map = [[false; 2048]; 2048];
            read_map(map, &format!("{MAPS_DIR}/{coleccion}/{map_name}"));
            for (escenario, line) in escenarios.into_iter().skip(1).enumerate().step_by(paso) {
                correr(map, decode_scen(line), &destino, escenario);
            }
            let minutos = inicio.elapsed().as_secs() / 60;
            anotar(format!("Finish map: {coleccion}/{map_name} ({} h {:02} min)", minutos / 60, minutos % 60));
        }
    }
}

fn run_map_struct_save<OpenType>(map: &CustomMap, stats: MapStats, destino: &str, escenario: usize, salida: &str, modo: Modo)
where OpenType: OpenList,
{
    //1. búsqueda limpia, cronometrada entera: es el tiempo total (execution_time). La open va
    //envuelta en Medido, que registra el máximo de memoria que reservó (su costo no se nota en el
    //tiempo), y en SinTamanos, para que calcular sus tamaños al llegar a la meta no entre en el
    //tiempo (ver memory_counter.rs)
    let star_time = Instant::now();
    let mut astar_data = a_star::<SinTamanos<Medido<OpenType>>, _>(
        stats.start,
        stats.goal,
        map,
        octile_distance
    ).unwrap();
    let execution_time = star_time.elapsed().as_nanos();
    let open_peak_bytes = astar_data.open.inner().peak_bytes();

    //2. solo en la corrida completa: la misma búsqueda otra vez, con cronómetro en cada operación de
    //la open (ver Cronometrado en open_time.rs). Su tiempo total no se usa: el cronómetro lo alarga
    let cronometro = if modo == Modo::Completa {
        //la tabla de nodos de la búsqueda limpia (varios MB) ya no se usa: su tamaño quedó en
        //close_bytes. Si siguiera viva, la segunda pasada tendría que pedir memoria nueva al sistema
        drop(std::mem::take(&mut astar_data.close));
        //lo mismo con la open de la búsqueda limpia (su máximo de memoria ya quedó guardado): así la
        //open cronometrada no paga pedirle memoria nueva al sistema
        drop(std::mem::take(&mut astar_data.open));
        let reloj_ns = calibrar_reloj();
        let cronometrada = a_star::<Cronometrado<OpenType>, _>(
            stats.start,
            stats.goal,
            map,
            octile_distance
        ).unwrap();
        //tiene que ser la misma búsqueda; si no, el tiempo de la open no le corresponde
        assert_eq!(astar_data.expansions, cronometrada.expansions, "la pasada con cronómetro no reprodujo la búsqueda de {}", OpenType::NAME);
        //los tamaños de la open al llegar a la meta salen de esta pasada: en la limpia, SinTamanos
        //los dejó en 0
        astar_data.open_bytes = cronometrada.open_bytes;
        astar_data.open_used_bytes = cronometrada.open_used_bytes;
        Some((cronometrada.open.ns(), cronometrada.open.intervalos(), reloj_ns))
    } else {
        None
    };

    //lo que no cambia entre corridas se escribe una sola vez, en la corrida completa
    if modo == Modo::Completa {
        write_files::escribir_busqueda(
            format!("{BUSQUEDAS}/{}/{destino}", OpenType::NAME),
            escenario,
            stats.start,
            stats.goal,
            stats.distance,
            &astar_data,
            open_peak_bytes,
        );
    }
    write_files::escribir_tiempo(
        format!("{salida}/{}/{destino}", OpenType::NAME),
        escenario,
        stats.start,
        stats.goal,
        astar_data.expansions,
        execution_time,
        cronometro,
    );
}

//corre las CORRIDAS completas, una después de otra. Una corrida terminada deja el archivo TERMINADA
//en su carpeta; al relanzar el programa se salta las terminadas y sigue la que quedó a medias desde
//el mapa en que iba
pub fn correr_corridas(map: &mut CustomMap) {
    for k in 1..=CORRIDAS {
        let salida = format!("{RESULTADOS}/corrida_{k}");
        let marca = format!("{salida}/TERMINADA");
        if Path::new(&marca).exists() {
            println!("Corrida {k}: ya estaba terminada");
            continue;
        }
        let modo = if k == 1 { Modo::Completa } else { Modo::SoloBusqueda };
        println!("Corrida {k} ({modo:?}) en {salida}");
        let inicio = Instant::now();
        test_structur_per_map(map, &salida, modo);
        let minutos = inicio.elapsed().as_secs() / 60;
        fs::write(&marca, format!("{modo:?}, {} h {:02} min desde el último lanzamiento\n", minutos / 60, minutos % 60))
            .expect("no se pudo escribir TERMINADA");
        println!("Corrida {k} terminada");
    }
}

pub fn test_structur_per_map(map: &mut CustomMap, salida: &str, modo: Modo) {
    //la corrida completa escribe también las búsquedas; las otras, solo sus tiempos
    let carpetas = match modo {
        Modo::Completa => vec![BUSQUEDAS.to_string(), salida.to_string()],
        Modo::SoloBusqueda => vec![salida.to_string()],
    };
    recorrer_escenarios(map, salida, &carpetas, |map, stats, destino, escenario| {
        run_map_struct_save::<BinaryHeapOpen>(map, stats, destino, escenario, salida, modo);
        run_map_struct_save::<RadixHeapOpen>(map, stats, destino, escenario, salida, modo);
        run_map_struct_save::<RadixAltRoundOpen>(map, stats, destino, escenario, salida, modo);
        run_map_struct_save::<VebOpen>(map, stats, destino, escenario, salida, modo);
        run_map_struct_save::<QuickHeapOpen>(map, stats, destino, escenario, salida, modo);
        run_map_struct_save::<QuickHeapScalarOpen>(map, stats, destino, escenario, salida, modo); // el mismo QuickHeap sin SIMD
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
