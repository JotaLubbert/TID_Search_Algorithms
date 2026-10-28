use std::mem::size_of;
use crate::a_star::{Coords, SearchNode};
use crate::closed_set::ClosedSet;

const NONE: u32 = u32::MAX;
const BRANCH_LINEAR_MAX: usize = 7;
const BRANCH_BITMAP_MAX: usize = 128;
const LEAF_LINEAR_MAX: usize = 8;
const PASO: usize = 8;

//reserva memoria completa para el vector, evita pedir memoria con cada inserción ahorrando tiempo
fn crecer<T>(v: &mut Vec<T>) {
    if v.len() == v.capacity() {
        v.reserve_exact(PASO);
    }
}

//comprueba si el bit está encendido
fn bit_has(bits: &[u64; 4], b: u8) -> bool {
    (bits[(b >> 6) as usize] >> (b & 63)) & 1 == 1
}
//enciende el bit usando la máscara con solo 1 bit encendido
fn bit_set(bits: &mut [u64; 4], b: u8) {
    bits[(b >> 6) as usize] |= 1u64 << (b & 63);
}

//cuenta los bits encendidos por debajo del bit b, indica la posición de puntero en el arreglo compacto
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
            //revisa si la rama lineal, si encuentra la key, retorna el puntero
            Branch::Linear { keys, ptrs, len } => {
                for i in 0..*len as usize {
                    if keys[i] == b {
                        return ptrs[i];
                    }
                }
                NONE
            }
            //si el bit está, retorna el puntero de la posición rank
            Branch::Bitmap { bits, ptrs } => {
                if bit_has(bits, b) { ptrs[rank(bits, b)] } else { NONE }
            }
            //indexa directamente
            Branch::Full(t) => t[b as usize],
        }
    }

    fn insert(&mut self, b: u8, p: u32) {
        match self {
            Branch::Linear { keys, ptrs, len } => {
                //si es lineal lo mete al final
                if (*len as usize) < BRANCH_LINEAR_MAX {
                    keys[*len as usize] = b;
                    ptrs[*len as usize] = p;
                    *len += 1;
                    return;
                }
                //pasa a bitmap, junta los pares y ordena por byte
                let mut pairs: Vec<(u8, u32)> = keys.iter().copied().zip(ptrs.iter().copied()).collect();
                pairs.push((b, p));
                pairs.sort_by_key(|x| x.0);

                //enciende un bit por hijo, arma el arreglo de punteros y reemplaza la rama por bitmap
                let mut bits = [0u64; 4];
                for (k, _) in &pairs {
                    bit_set(&mut bits, *k);
                }
                let mut v: Vec<u32> = Vec::with_capacity(pairs.len());
                v.extend(pairs.iter().map(|x| x.1));
                *self = Branch::Bitmap { bits, ptrs: v };
            }
            Branch::Bitmap { bits, ptrs } => {
                //si está lleno pasa a full y copia cada puntero a su casilla usando rank
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
                //calcula el rank del bit y lo inserta en la posición
                let r = rank(bits, b);
                crecer(ptrs);
                ptrs.insert(r, p);
                bit_set(bits, b);
            }
            Branch::Full(t) => t[b as usize] = p,
        }
    }

    //memoria dinámica de la rama
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
            //recorre comparando coordenadas, retorna el nodo si encuentra
            Leaf::Linear(v) => v.iter().find(|n| n.coords == c),
            //comprueba si el bit está encendido, retorna el nodo de la posición rank
            Leaf::Bitmap { bits, nodes } => {
                let b = low(c);
                if bit_has(bits, b) { Some(&nodes[rank(bits, b)]) } else { None }
            }
        }
    }

    fn insert(&mut self, node: SearchNode) {
        match self {
            Leaf::Linear(v) => {
                //si encuentra el nodo en el vector lo actualiza
                if let Some(n) = v.iter_mut().find(|n| n.coords == node.coords) {
                    *n = node;
                    return;
                }
                //si no hay espacio pasa a bitmap
                if v.len() < LEAF_LINEAR_MAX {
                    crecer(v);
                    v.push(node);
                    return;
                }
                let mut all = std::mem::take(v);
                all.reserve_exact(PASO);
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
                    //crecer el vector de los nodos si se llena
                    crecer(nodes);
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

//crea la clave de 22 bits a partir de las coordenadas
fn split(c: Coords) -> (u8, u8) {
    debug_assert!(c.0 < 2048 && c.1 < 2048);
    let k = (c.1 << 11) | c.0;
    (((k >> 16) & 0xFF) as u8, ((k >> 8) & 0xFF) as u8)
}

impl ClosedSet for Judy {
    //crea la raíz y los vectores de rama y hoja
    fn new() -> Self {
        Judy { root: Branch::new(), branches: Vec::new(), leaves: Vec::new() }
    }

    //para buscar baja por los tres niveles, raíz da el índice de la rama, la rama el índice de la hoja y la hoja el nodo, retorna none si no encuentra en alguno de los niveles
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

    //también baja por los niveles, si no encuentra rama o hoja la crea
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