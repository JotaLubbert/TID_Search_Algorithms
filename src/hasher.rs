use std::hash::{BuildHasherDefault, Hasher};

use crate::robin_hood::hash_coords;

#[derive(Default)]
pub struct CoordHasher {
    x: u32,
    y: u32,
    n: u8,
}

impl Hasher for CoordHasher {
    #[inline]
    fn write_u32(&mut self, v: u32) {
        if self.n == 0 { self.x = v; } else { self.y = v; }
        self.n += 1;
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("solo se usa con claves (u32, u32)")
    }

    #[inline]
    fn finish(&self) -> u64 {
        let h = hash_coords((self.x, self.y)) as u64;
        h | (h << 32)
    }
}

pub type CoordBuild = BuildHasherDefault<CoordHasher>;