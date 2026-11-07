use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::a_star::{a_star, Coords, SearchNode};
use crate::closed_set::ClosedSet;
use crate::distances_types::euclidean_distance;
use crate::hasher::CoordBuild;
use crate::judy::Judy;
use crate::read_files::{decode_scen, read_lines, read_map, MapStats};
use crate::robin_hood::RobinHood;
use crate::testing_functions::carpetas_grupo;
use crate::CustomMap;

// HashMap con SipHash (baseline) y HashMap con el hash de Robin Hood
type HmSip = HashMap<Coords, SearchNode>;
type HmRobin = HashMap<Coords, SearchNode, CoordBuild>;

const NOMBRES: [&str; 4] = ["hashmap", "hashmap_hash_robin", "robin_hood", "judy"];
const N: usize = 4;

// ---------- generador determinista: las estructuras reciben exactamente las mismas claves ----------
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
    // t[estructura][operacion] = lista de repeticiones
    let mut t = vec![vec![Vec::new(); 3]; N];
    for rep in 0..=reps {
        for k in 0..N {
            let idx = (k + rep) % N; // se rota el orden: ninguna estructura va siempre primero
            let r = match idx {
                0 => micro_una::<HmSip>(keys, misses),
                1 => micro_una::<HmRobin>(keys, misses),
                2 => micro_una::<RobinHood>(keys, misses),
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
    println!(
        "\n[{}]  {} claves, {} consultas fallidas, mediana de {} repeticiones",
        nombre,
        keys.len(),
        misses.len(),
        reps
    );
    // med[estructura][operacion] = mediana en ns totales
    let med: Vec<Vec<f64>> = (0..N)
        .map(|i| (0..3).map(|o| mediana(&mut t[i][o].clone()) as f64).collect())
        .collect();
    let ops = [keys.len() as f64, keys.len() as f64, misses.len() as f64];
    for refi in [0usize, 1usize] {
        println!("referencia: {}", NOMBRES[refi]);
        println!("{:<20} {:>16} {:>16} {:>16}", "estructura", "insert (ns/op)", "get acierto", "get fallo");
        for i in 0..N {
            let celda = |o: usize| {
                let ns = med[i][o] / ops[o];
                if i == refi {
                    format!("{:.1}", ns)
                } else {
                    format!("{:.1} ({:+.0}%)", ns, (med[i][o] / med[refi][o] - 1.0) * 100.0)
                }
            };
            println!("{:<20} {:>16} {:>16} {:>16}", NOMBRES[i], celda(0), celda(1), celda(2));
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

fn una_corrida<C: ClosedSet>(map: &CustomMap, s: &MapStats) -> Option<(u128, f64, u64, u64)> {
    let t = Instant::now();
    let r = a_star::<C, _>(s.start, s.goal, map, euclidean_distance)?;
    let ns = t.elapsed().as_nanos();
    black_box(&r.path);
    Some((ns, r.final_dis, r.expansions, r.generated))
}

pub fn bench_directo(map: &mut CustomMap, grupo: &str, scen_name: &str, reps: usize, max_escenarios: usize) {
    let (scen_dir, maps_dir) = carpetas_grupo(grupo);
    let map_name = scen_name.strip_suffix(".scen").expect("el nombre debe terminar en .scen");
    read_map(map, &format!("{}/{}", maps_dir, map_name));
    let lines: Vec<String> = read_lines(&format!("{}/{}", scen_dir, scen_name))
        .into_iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .collect();
    // se reparten los escenarios usados a lo largo de todo el archivo
    let paso = (lines.len() / max_escenarios.max(1)).max(1);
    let escenarios: Vec<MapStats> = lines.into_iter().step_by(paso).take(max_escenarios).map(decode_scen).collect();

    let mut total = [0u128; N];
    let mut tsv = String::from("start_x\tstart_y\tgoal_x\tgoal_y\texpansions");
    for n in NOMBRES {
        tsv.push_str(&format!("\t{}_ns", n));
    }
    tsv.push('\n');
    let mut distintos = 0;
    let mut usados = 0;
    for s in &escenarios {
        let mut mejor = [u128::MAX; N];
        let mut datos: [Option<(f64, u64, u64)>; N] = [None; N];
        let mut ok = true;
        for rep in 0..=reps {
            for k in 0..N {
                let idx = (k + rep) % N; // orden rotado
                let r = match idx {
                    0 => una_corrida::<HmSip>(map, s),
                    1 => una_corrida::<HmRobin>(map, s),
                    2 => una_corrida::<RobinHood>(map, s),
                    _ => una_corrida::<Judy>(map, s),
                };
                match r {
                    Some((ns, d, e, g)) => {
                        if rep > 0 {
                            // el mínimo es lo menos afectado por ruido del sistema
                            mejor[idx] = mejor[idx].min(ns);
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
        for i in 1..N {
            let (d, e, g) = datos[i].unwrap();
            if e != e0 || g != g0 || (d - d0).abs() > 1e-9 {
                distintos += 1;
            }
        }
        for i in 0..N {
            total[i] += mejor[i];
        }
        tsv.push_str(&format!("{}\t{}\t{}\t{}\t{}", s.start.0, s.start.1, s.goal.0, s.goal.1, e0));
        for i in 0..N {
            tsv.push_str(&format!("\t{}", mejor[i]));
        }
        tsv.push('\n');
    }
    let _ = std::fs::create_dir_all("test_result");
    let _ = std::fs::write(format!("test_result/bench_directo_{}_{}.tsv", grupo, map_name), tsv);

    println!(
        "\n[A* completo] grupo {} · mapa {}  |  {} escenarios, mínimo de {} repeticiones por escenario",
        grupo, map_name, usados, reps
    );
    println!("{:<20} {:>12} {:>14} {:>22}", "estructura", "total (ms)", "vs hashmap", "vs hashmap_hash_robin");
    for i in 0..N {
        let ms = total[i] as f64 / 1e6;
        let rel = |r: usize| {
            if i == r {
                "base".to_string()
            } else {
                format!("{:+.1} %", (total[i] as f64 / total[r] as f64 - 1.0) * 100.0)
            }
        };
        println!("{:<20} {:>12.1} {:>14} {:>22}", NOMBRES[i], ms, rel(0), rel(1));
    }
    println!(
        "expansiones, generados y distancia idénticos entre estructuras: {}",
        if distintos == 0 { "SÍ".to_string() } else { format!("NO ({} diferencias)", distintos) }
    );
}