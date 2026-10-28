use std::mem::size_of;
use crate::a_star::{Coords, SearchNode};
use crate::closed_set::ClosedSet;

// dist = 0 -> casilla vacía; si no, distancia al bucket ideal + 1
#[derive(Clone, Copy)]
struct Slot {
    node: SearchNode,
    dist: u32,
}

const EMPTY: Slot = Slot {
    node: SearchNode { coords: (0, 0), g: 0.0, _h: 0.0, f: 0.0, parent: None },
    dist: 0,
};

pub struct RobinHood {
    slots: Vec<Slot>,
    len: usize,
}

#[inline]
fn hash(c: Coords) -> usize {
    let k = ((c.1 as u64) << 32) | c.0 as u64;
    (k.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) as usize
}

impl RobinHood {
    fn find(&self, c: Coords) -> Option<usize> {
        let mask = self.slots.len() - 1;
        let mut idx = hash(c) & mask;
        let mut dist = 1;
        loop {
            let s = &self.slots[idx];
            // vacía, o su dist es menor
            if s.dist == 0 || s.dist < dist {
                return None;
            }
            if s.node.coords == c {
                return Some(idx);
            }
            idx = (idx + 1) & mask;
            dist += 1;
        }
    }

    // inserta un nodo que se sabe que no existe
    fn insert_new(&mut self, node: SearchNode) {
        let mask = self.slots.len() - 1;
        let mut idx = hash(node.coords) & mask;
        let mut carry = Slot { node, dist: 1 };
        loop {
            let s = &mut self.slots[idx];
            if s.dist == 0 {
                *s = carry;
                self.len += 1;
                return;
            }
            // intercambio de nodos
            if s.dist < carry.dist {
                std::mem::swap(s, &mut carry);
            }
            idx = (idx + 1) & mask;
            carry.dist += 1;
        }
    }

    fn grow(&mut self) {
        let new_len = self.slots.len() * 2;
        let old = std::mem::replace(&mut self.slots, vec![EMPTY; new_len]);
        self.len = 0;
        for s in old {
            if s.dist != 0 {
                self.insert_new(s.node);
            }
        }
    }
}

impl ClosedSet for RobinHood {
    fn new() -> Self {
        RobinHood { slots: vec![EMPTY; 16], len: 0 }
    }

    fn get(&self, c: Coords) -> Option<&SearchNode> {
        self.find(c).map(|i| &self.slots[i].node)
    }

    fn insert(&mut self, node: SearchNode) {
        if let Some(i) = self.find(node.coords) {
            self.slots[i].node = node; // actualiza g y parent
            return;
        }
        // factor de carga máximo 7/8
        if (self.len + 1) * 8 > self.slots.len() * 7 {
            self.grow();
        }
        self.insert_new(node);
    }

    fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + self.slots.capacity() * size_of::<Slot>()
    }
    fn len(&self) -> usize { self.len }
}