use std::cell::Cell;
use std::time::Instant;

use crate::a_star::{Coords, SearchNode};
use crate::closed_set::ClosedSet;

// acumulado de una corrida de A*: tiempo y cantidad de operaciones sobre el Closed
#[derive(Clone, Copy, Default, Debug)]
pub struct ClosedStats {
    pub get_ns: u64,
    pub gets: u64,
    pub insert_ns: u64,
    pub inserts: u64,
}

thread_local! {
    static STATS: Cell<ClosedStats> = Cell::new(ClosedStats::default());
    static SOBRECOSTO_NS: Cell<f64> = Cell::new(0.0);
}

pub fn reset_stats() {
    STATS.with(|s| s.set(ClosedStats::default()));
}

// devuelve los tiempos con el costo del propio cronómetro ya descontado
pub fn take_stats() -> ClosedStats {
    let raw = STATS.with(|s| s.get());
    let o = SOBRECOSTO_NS.with(|o| o.get());
    ClosedStats {
        get_ns: (raw.get_ns as f64 - o * raw.gets as f64).max(0.0) as u64,
        gets: raw.gets,
        insert_ns: (raw.insert_ns as f64 - o * raw.inserts as f64).max(0.0) as u64,
        inserts: raw.inserts,
    }
}

// mide cuánto marca el cronómetro cuando no hay nada entre las dos lecturas
pub fn calibrar() -> f64 {
    let n = 2_000_000u64;
    let mut total = 0u64;
    for _ in 0..n {
        let t = Instant::now();
        total += t.elapsed().as_nanos() as u64;
    }
    let o = total as f64 / n as f64;
    SOBRECOSTO_NS.with(|c| c.set(o));
    o
}

// envuelve cualquier Closed y cronometra cada get e insert
pub struct Timed<C: ClosedSet> {
    inner: C,
}

impl<C: ClosedSet> ClosedSet for Timed<C> {
    fn new() -> Self {
        Timed { inner: C::new() }
    }

    fn get(&self, c: Coords) -> Option<&SearchNode> {
        let t = Instant::now();
        let r = self.inner.get(c);
        let dt = t.elapsed().as_nanos() as u64;
        STATS.with(|s| {
            let mut v = s.get();
            v.get_ns += dt;
            v.gets += 1;
            s.set(v);
        });
        r
    }

    fn insert(&mut self, node: SearchNode) {
        let t = Instant::now();
        self.inner.insert(node);
        let dt = t.elapsed().as_nanos() as u64;
        STATS.with(|s| {
            let mut v = s.get();
            v.insert_ns += dt;
            v.inserts += 1;
            s.set(v);
        });
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn size_in_bytes(&self) -> usize {
        self.inner.size_in_bytes()
    }
}