use std::mem::size_of;
use crate::a_star::{Coords, SearchNode};
use crate::closed_set::ClosedSet;

const NONE: u32 = u32::MAX;
const BRANCH_LINEAR_MAX: usize = 7;
const BRANCH_BITMAP_MAX: usize = 128;
const LEAF_LINEAR_MAX: usize = 8;

fn bit_has(bits: &[u64; 4], b: u8) -> bool {
    (bits[(b >> 6) as usize] >> (b & 63)) & 1 == 1
}
fn bit_set(bits: &mut [u64; 4], b: u8) {
    bits[(b >> 6) as usize] |= 1u64 << (b & 63);
}
fn rank(bits: &[u64; 4], b: u8) -> usize {
    let w = (b >> 6) as usize;
    let mut r = 0;
    for i in 0..w {
        r += bits[i].count_ones() as usize;
    }
    r + (bits[w] & ((1u64 << (b & 63)) - 1)).count_ones() as usize
}

enum Branch {
    Linear { keys: [u8; BRANCH_LINEAR_MAX], ptrs: [u32; BRANCH_LINEAR_MAX], len: u8 },
    Bitmap { bits: [u64; 4], ptrs: Vec<u32> },
    Full(Box<[u32; 256]>),
}

impl Branch {
    fn new() -> Self {
        Branch::Linear { keys: [0; BRANCH_LINEAR_MAX], ptrs: [NONE; BRANCH_LINEAR_MAX], len: 0 }
    }

    fn get(&self, b: u8) -> u32 {
        match self {
            Branch::Linear { keys, ptrs, len } => {
                for i in 0..*len as usize {
                    if keys[i] == b {
                        return ptrs[i];
                    }
                }
                NONE
            }
            Branch::Bitmap { bits, ptrs } => {
                if bit_has(bits, b) { ptrs[rank(bits, b)] } else { NONE }
            }
            Branch::Full(t) => t[b as usize],
        }
    }

    fn insert(&mut self, b: u8, p: u32) {
        match self {
            Branch::Linear { keys, ptrs, len } => {
                if (*len as usize) < BRANCH_LINEAR_MAX {
                    keys[*len as usize] = b;
                    ptrs[*len as usize] = p;
                    *len += 1;
                    return;
                }
                let mut pairs: Vec<(u8, u32)> = keys.iter().copied().zip(ptrs.iter().copied()).collect();
                pairs.push((b, p));
                pairs.sort_by_key(|x| x.0);
                let mut bits = [0u64; 4];
                for (k, _) in &pairs {
                    bit_set(&mut bits, *k);
                }
                let mut v: Vec<u32> = Vec::with_capacity(pairs.len());
                v.extend(pairs.iter().map(|x| x.1));
                *self = Branch::Bitmap { bits, ptrs: v };
            }
            Branch::Bitmap { bits, ptrs } => {
                if ptrs.len() >= BRANCH_BITMAP_MAX {
                    let mut t = Box::new([NONE; 256]);
                    for k in 0..=255u8 {
                        if bit_has(bits, k) {
                            t[k as usize] = ptrs[rank(bits, k)];
                        }
                    }
                    t[b as usize] = p;
                    *self = Branch::Full(t);
                    return;
                }
                let r = rank(bits, b);
                ptrs.reserve_exact(1);
                ptrs.insert(r, p);
                bit_set(bits, b);
            }
            Branch::Full(t) => t[b as usize] = p,
        }
    }

    fn heap_bytes(&self) -> usize {
        match self {
            Branch::Linear { .. } => 0,
            Branch::Bitmap { ptrs, .. } => ptrs.capacity() * size_of::<u32>(),
            Branch::Full(_) => size_of::<[u32; 256]>(),
        }
    }
}

enum Leaf {
    Linear(Vec<SearchNode>),
    Bitmap { bits: [u64; 4], nodes: Vec<SearchNode> },
}

fn low(c: Coords) -> u8 {
    (c.0 & 0xFF) as u8
}

impl Leaf {
    fn get(&self, c: Coords) -> Option<&SearchNode> {
        match self {
            Leaf::Linear(v) => v.iter().find(|n| n.coords == c),
            Leaf::Bitmap { bits, nodes } => {
                let b = low(c);
                if bit_has(bits, b) { Some(&nodes[rank(bits, b)]) } else { None }
            }
        }
    }

    fn insert(&mut self, node: SearchNode) {
        match self {
            Leaf::Linear(v) => {
                if let Some(n) = v.iter_mut().find(|n| n.coords == node.coords) {
                    *n = node;
                    return;
                }
                if v.len() < LEAF_LINEAR_MAX {
                    v.reserve_exact(1);
                    v.push(node);
                    return;
                }
                let mut all = std::mem::take(v);
                all.reserve_exact(1);
                all.push(node);
                all.sort_by_key(|n| low(n.coords));
                let mut bits = [0u64; 4];
                for n in &all {
                    bit_set(&mut bits, low(n.coords));
                }
                *self = Leaf::Bitmap { bits, nodes: all };
            }
            Leaf::Bitmap { bits, nodes } => {
                let b = low(node.coords);
                let r = rank(bits, b);
                if bit_has(bits, b) {
                    nodes[r] = node;
                } else {
                    nodes.reserve_exact(1);
                    nodes.insert(r, node);
                    bit_set(bits, b);
                }
            }
        }
    }

    fn heap_bytes(&self) -> usize {
        match self {
            Leaf::Linear(v) => v.capacity() * size_of::<SearchNode>(),
            Leaf::Bitmap { nodes, .. } => nodes.capacity() * size_of::<SearchNode>(),
        }
    }
    fn len(&self) -> usize {
        match self {
            Leaf::Linear(v) => v.len(),
            Leaf::Bitmap { nodes, .. } => nodes.len(),
        }
    }
}


pub struct Judy {
    root: Branch,
    branches: Vec<Branch>,
    leaves: Vec<Leaf>,
}

fn split(c: Coords) -> (u8, u8) {
    debug_assert!(c.0 < 2048 && c.1 < 2048);
    let k = (c.1 << 11) | c.0;
    (((k >> 16) & 0xFF) as u8, ((k >> 8) & 0xFF) as u8)
}

impl ClosedSet for Judy {
    fn new() -> Self {
        Judy { root: Branch::new(), branches: Vec::new(), leaves: Vec::new() }
    }

    fn get(&self, c: Coords) -> Option<&SearchNode> {
        let (b2, b1) = split(c);
        let bi = self.root.get(b2);
        if bi == NONE {
            return None;
        }
        let li = self.branches[bi as usize].get(b1);
        if li == NONE {
            return None;
        }
        self.leaves[li as usize].get(c)
    }

    fn insert(&mut self, node: SearchNode) {
        let (b2, b1) = split(node.coords);
        let mut bi = self.root.get(b2);
        if bi == NONE {
            bi = self.branches.len() as u32;
            self.branches.push(Branch::new());
            self.root.insert(b2, bi);
        }
        let mut li = self.branches[bi as usize].get(b1);
        if li == NONE {
            li = self.leaves.len() as u32;
            self.leaves.push(Leaf::Linear(Vec::new()));
            self.branches[bi as usize].insert(b1, li);
        }
        self.leaves[li as usize].insert(node);
    }

    fn size_in_bytes(&self) -> usize {
        size_of::<Self>()
            + self.root.heap_bytes()
            + self.branches.capacity() * size_of::<Branch>()
            + self.branches.iter().map(|b| b.heap_bytes()).sum::<usize>()
            + self.leaves.capacity() * size_of::<Leaf>()
            + self.leaves.iter().map(|l| l.heap_bytes()).sum::<usize>()
    }
    fn len(&self) -> usize { self.leaves.iter().map(|l| l.len()).sum() }
}
