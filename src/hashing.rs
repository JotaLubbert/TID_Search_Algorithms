use std::hash::{BuildHasherDefault, Hasher};
use crate::a_star::Coords;

#[inline(always)]
pub fn hash64(c: Coords) -> u64 {
    let k = ((c.1 as u64) << 32) | c.0 as u64;
    let h = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 32)
}

#[derive(Default)]
pub struct CoordHasher { x: u32, y: u32, n: u8 }

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
    fn finish(&self) -> u64 { hash64((self.x, self.y)) }
}

pub type CoordBuild = BuildHasherDefault<CoordHasher>;