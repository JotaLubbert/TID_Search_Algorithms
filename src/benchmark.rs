use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::CustomMap;
use crate::a_star::{a_star, Coords, SearchNode};
use crate::closed_set::ClosedSet;
use crate::distances_types::euclidean_distance;
use crate::judy::Judy;
use crate::read_files::{decode_scen, read_lines, read_map, MapStats};
use crate::robin_hood::RobinHood;

const NOMBRES: [&str; 3] = ["hashmap", "robin_hood", "judy"];

struct Xor(u64);
impl Xor {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn mediana(v: &mut Vec<u128>) -> u128 {
    v.sort();
    v[v.len() / 2]
}


// solo las operaciones del Closed (sin A*, sin heap)
fn micro_una<C: ClosedSet>(keys: &[Coords], misses: &[Coords]) -> (u128, u128, u128) {
    let t = Instant::now();
    let mut c = C::new();
    for &k in keys {
        c.insert(SearchNode::new(k, 1.0, 0.0, None));
    }
    let ins = t.elapsed().as_nanos();

    let t = Instant::now();
    let mut acc = 0.0;
    for &k in keys {
        acc += c.get(k).map_or(0.0, |n| n.g);
    }
    let hit = t.elapsed().as_nanos();

    let t = Instant::now();
    for &k in misses {
        if c.get(k).is_some() {
            acc += 1.0;
        }
    }
    let miss = t.elapsed().as_nanos();

    black_box(acc);
    black_box(&c);
    (ins, hit, miss)
}

fn micro_patron(nombre: &str, keys: &[Coords], misses: &[Coords], reps: usize) {
    // tiempos[estructura][operacion] = lista de repeticiones
    let mut t = vec![vec![Vec::new(); 3]; 3];
    for rep in 0..=reps {
        for k in 0..3 {
            let idx = (k + rep) % 3; // se rota el orden para que ninguna estructura siempre vaya primero
            let r = match idx {
                0 => micro_una::<HashMap<Coords, SearchNode>>(keys, misses),
                1 => micro_una::<RobinHood>(keys, misses),
                _ => micro_una::<Judy>(keys, misses),
            };
            if rep > 0 {
                // la repetición 0 es de calentamiento y no se cuenta
                t[idx][0].push(r.0);
                t[idx][1].push(r.1);
                t[idx][2].push(r.2);
            }
        }
    }
    println!("\n[{}]  {} claves, {} consultas fallidas, mediana de {} repeticiones", nombre, keys.len(), misses.len(), reps);
    println!("{:<12} {:>14} {:>14} {:>14}", "estructura", "insert (ns/op)", "get acierto", "get fallo");
    let base: Vec<f64> = (0..3).map(|o| mediana(&mut t[0][o].clone()) as f64).collect();
    for i in 0..3 {
        let m: Vec<u128> = (0..3).map(|o| mediana(&mut t[i][o].clone())).collect();
        let nsop = [m[0] as f64 / keys.len() as f64, m[1] as f64 / keys.len() as f64, m[2] as f64 / misses.len() as f64];
        let pct = |o: usize| (m[o] as f64 - base[o]) / base[o] * 100.0;
        if i == 0 {
            println!("{:<12} {:>14.1} {:>14.1} {:>14.1}", NOMBRES[i], nsop[0], nsop[1], nsop[2]);
        } else {
            println!("{:<12} {:>7.1} ({:+.0}%) {:>7.1} ({:+.0}%) {:>7.1} ({:+.0}%)", NOMBRES[i], nsop[0], pct(0), nsop[1], pct(1), nsop[2], pct(2));
        }
    }
}

pub fn micro_directo() {
    let n = 400_000usize;
    let mut r = Xor(0x1234_5678_9ABC_DEF1);
    // patrón A: claves aleatorias en todo el mapa (peor caso de localidad)
    let keys_a: Vec<Coords> = (0..n).map(|_| ((r.next() % 2048) as u32, (r.next() % 2048) as u32)).collect();
    let miss_a: Vec<Coords> = (0..n).map(|_| ((r.next() % 2048) as u32, (r.next() % 2048) as u32)).collect();
    micro_patron("claves aleatorias", &keys_a, &miss_a, 5);

    // patrón B: frente de onda en diagonales, parecido a cómo A* va explorando
    let lado = 640u32;
    let mut keys_b = Vec::new();
    for d in 0..(2 * lado - 1) {
        let x0 = d.saturating_sub(lado - 1);
        let x1 = d.min(lado - 1);
        for x in x0..=x1 {
            keys_b.push((x, d - x));
        }
    }
    let miss_b: Vec<Coords> = keys_b.iter().map(|&(x, y)| (x + lado, y)).collect(); // justo fuera de la región
    micro_patron("frente de onda", &keys_b, &miss_b, 5);
}


// las tres estructuras sobre los mismos escenarios, intercaladas
fn una_corrida<C: ClosedSet>(map: &CustomMap, s: &MapStats) -> Option<(u128, f64, u64, u64)> {
    let t = Instant::now();
    let r = a_star::<C, _>(s.start, s.goal, map, euclidean_distance)?;
    let ns = t.elapsed().as_nanos();
    black_box(&r.path);
    Some((ns, r.final_dis, r.expansions, r.generated))
}

pub fn bench_directo(map: &mut CustomMap, scen_name: &str, reps: usize, max_escenarios: usize) {
    let map_name = scen_name.strip_suffix(".scen").expect("el nombre debe terminar en .scen");
    *map = [[false; 2048]; 2048];
    read_map(map, &format!("maps/{}", map_name));
    let lines: Vec<String> = read_lines(&format!("test_data/{}", scen_name))
        .into_iter().skip(1).filter(|l| !l.trim().is_empty()).collect();
    // se reparten los escenarios usados a lo largo de todo el archivo
    let paso = (lines.len() / max_escenarios.max(1)).max(1);
    let escenarios: Vec<MapStats> = lines.into_iter().step_by(paso).take(max_escenarios).map(decode_scen).collect();

    let mut total = [0u128; 3];
    let mut tsv = String::from("start_x\tstart_y\tgoal_x\tgoal_y\texpansions\thashmap_ns\trobin_hood_ns\tjudy_ns\n");
    let mut distintos = 0;
    let mut usados = 0;
    for s in &escenarios {
        let mut mejor = [u128::MAX; 3];
        let mut datos: [Option<(f64, u64, u64)>; 3] = [None; 3];
        let mut ok = true;
        for rep in 0..=reps {
            for k in 0..3 {
                let idx = (k + rep) % 3;
                let r = match idx {
                    0 => una_corrida::<HashMap<Coords, SearchNode>>(map, s),
                    1 => una_corrida::<RobinHood>(map, s),
                    _ => una_corrida::<Judy>(map, s),
                };
                match r {
                    Some((ns, d, e, g)) => {
                        if rep > 0 {
                            mejor[idx] = mejor[idx].min(ns); // se queda con el mínimo: el menos afectado por ruido del sistema
                        }
                        datos[idx] = Some((d, e, g));
                    }
                    None => ok = false,
                }
            }
        }
        if !ok {
            continue;
        }
        usados += 1;
        let (d0, e0, g0) = datos[0].unwrap();
        for i in 1..3 {
            let (d, e, g) = datos[i].unwrap();
            if e != e0 || g != g0 || (d - d0).abs() > 1e-9 {
                distintos += 1;
            }
        }
        for i in 0..3 {
            total[i] += mejor[i];
        }
        tsv.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", s.start.0, s.start.1, s.goal.0, s.goal.1, e0, mejor[0], mejor[1], mejor[2]));
    }
    let _ = std::fs::create_dir_all("test_result");
    let _ = std::fs::write(format!("test_result/bench_directo_{}.tsv", map_name), tsv);

    println!("\n[A* completo] mapa {}  |  {} escenarios, mínimo de {} repeticiones por escenario", map_name, usados, reps);
    println!("{:<12} {:>12} {:>16}", "estructura", "total (ms)", "vs hashmap");
    for i in 0..3 {
        let ms = total[i] as f64 / 1e6;
        let rel = if i == 0 { "base".to_string() } else { format!("{:+.1} %", (total[i] as f64 - total[0] as f64) / total[0] as f64 * 100.0) };
        println!("{:<12} {:>12.1} {:>16}", NOMBRES[i], ms, rel);
    }
    println!("expansiones, generados y distancia idénticos entre estructuras: {}", if distintos == 0 { "SÍ".to_string() } else { format!("NO ({} diferencias)", distintos) });
}