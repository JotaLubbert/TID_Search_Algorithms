use std::fs::{self, OpenOptions};
use std::io::{Write};
use std::path::Path;

use crate::{a_star::{AStarResults, Coords}, open_list::OpenList};

//Los resultados van en dos tipos de .tsv, uno por mapa, estructura y tipo (ver correr_corridas en
//testing_functions.rs):
//- de búsqueda: lo que no cambia entre corridas (el camino, cuánto trabajó A*, la memoria). Se
//  escribe una vez, en la corrida 1
//- de tiempo: lo que se mide de nuevo en cada corrida
//Los dos empiezan con escenario (la línea del escenario en el .scen, contando desde 0 sin la de
//la versión), el inicio y la meta, que identifican la búsqueda para cruzarlos en Python.
//En los dos, las columnas van en el mismo orden que se escriben las filas.

//path va al final a proposito: es la unica de ancho variable, asi que dejarla ahi evita que el
//resto de las columnas se desalinee
const HEADER_BUSQUEDA: &str = "escenario\tstart_x\tstart_y\tgoal_x\tgoal_y\texpected_distance\tactual_distance\tpath_len\texpansions\tgenerated\topen_bytes\topen_used_bytes\topen_peak_bytes\tclose_bytes\tpath";
//expansions se repite para comprobar que cada corrida hizo la misma búsqueda
const HEADER_TIEMPO: &str = "escenario\tstart_x\tstart_y\tgoal_x\tgoal_y\texpansions\texecution_time";
//la corrida completa agrega la pasada con cronómetro
const HEADER_TIEMPO_CRONOMETRO: &str = "escenario\tstart_x\tstart_y\tgoal_x\tgoal_y\texpansions\texecution_time\topen_timer_ns\topen_intervals\tclock_overhead_ns";

//ruta es la ruta del archivo sin el .tsv, con la carpeta que corresponda
//(ej. "test_result/busquedas/binary-heap/DragonAges/arena.map.scen")
pub fn escribir_busqueda(
    ruta: String,
    escenario: usize,
    start: Coords,
    goal: Coords,
    expected_distance: f64,
    results: &AStarResults<impl OpenList>,
    //máximo que reservó la open en la búsqueda (lo mide Medido, A* no lo conoce)
    open_peak_bytes: usize,
) {
    //el camino completo, horizontal: "(x, y) (x, y) ...". Sin tabs ni saltos de
    //linea, que son los unicos caracteres que romperian el tsv.
    let mut path_str = String::with_capacity(results.path.len() * 12);
    for (i, coord) in results.path.iter().enumerate() {
        if i > 0 {
            path_str.push(' ');
        }
        path_str.push_str(&format!("({}, {})", coord.0, coord.1));
    }

    let fila = format!(
        "{}\t{}\t{}\t{}\t{}\t{:.8}\t{:.8}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        escenario,
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
        path_str
    );
    agregar_fila(ruta, HEADER_BUSQUEDA, &fila);
}

//ruta como en escribir_busqueda (ej. "test_result/corrida_2/binary-heap/DragonAges/arena.map.scen")
pub fn escribir_tiempo(
    ruta: String,
    escenario: usize,
    start: Coords,
    goal: Coords,
    expansions: u64,
    //la búsqueda limpia completa, en ns
    execution_time: u128,
    //la pasada con cronómetro (ver open_time.rs), solo en la corrida completa: suma cruda de los
    //intervalos en ns, cuántos intervalos fueron y cuánto mide un intervalo vacío en ns. El tiempo
    //de la open es open_timer_ns − open_intervals × clock_overhead_ns
    cronometro: Option<(u64, u64, f64)>,
) {
    let comunes = format!("{escenario}\t{}\t{}\t{}\t{}\t{expansions}\t{execution_time}", start.0, start.1, goal.0, goal.1);
    match cronometro {
        Some((ns, intervalos, reloj_ns)) => {
            agregar_fila(ruta, HEADER_TIEMPO_CRONOMETRO, &format!("{comunes}\t{ns}\t{intervalos}\t{reloj_ns:.3}"))
        }
        None => agregar_fila(ruta, HEADER_TIEMPO, &comunes),
    }
}

//agrega una fila al .tsv; si el archivo no existe, lo crea con su encabezado (y su carpeta).
//Si no puede escribir (disco lleno, por ejemplo) detiene el programa en vez de perder filas sin
//avisar; al relanzarlo, la corrida rehace el mapa que quedó incompleto
fn agregar_fila(mut ruta: String, header: &str, fila: &str) {
    ruta.push_str(".tsv");
    if let Some(carpeta) = Path::new(&ruta).parent() {
        fs::create_dir_all(carpeta).expect("no se pudo crear la carpeta de resultados");
    }
    if !Path::new(&ruta).exists() {
        fs::write(&ruta, format!("{}\n", header)).unwrap_or_else(|e| panic!("no se pudo crear {ruta}: {e}"));
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(&ruta)
        .unwrap_or_else(|e| panic!("no se pudo abrir {ruta}: {e}"));
    writeln!(file, "{}", fila).unwrap_or_else(|e| panic!("no se pudo escribir en {ruta}: {e}"));
}
