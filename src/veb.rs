use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{BuildHasherDefault, Hasher};
use std::mem::{self, size_of};

use crate::a_star::hashmap_heap_bytes;

//Árbol de van Emde Boas sobre claves u64, pensado para guardar f.to_bits() en la open de A*.
//Es un conjunto: cada clave está una sola vez, los empates de f se manejan afuera.
//
//Decisiones para tiempo de ejecución:
//- Los clusters se crean solo cuando reciben un elemento y viven en un HashMap, así la
//  memoria depende de cuántas claves hay y no del tamaño del universo (2^64).
//- Ese HashMap usa un hash rápido para enteros: el SipHash por defecto de Rust es lento
//  para claves u64, y aquí hay una búsqueda por nivel.
//- Los clusters van guardados directo dentro del HashMap, sin un Box por cluster: con
//  claves f.to_bits() casi todos los clusters tienen un solo elemento, y así crearlos
//  no pide memoria al allocator. Solo el summary va en Box (si no, el tipo sería infinito).
//- Los universos de 8 bits o menos son hojas: un bitset de 256 bits ([u64; 4]). Con claves
//  de 64 bits la recursión queda 64 -> 32 -> 16 -> hoja de 8, o sea tres búsquedas en
//  HashMap por operación, en vez de seguir bajando hasta universos de 2 bits.
//- Como en CLRS, el mínimo de cada nodo no se guarda en sus clusters: insertar en un nodo
//  vacío es O(1) y cada operación baja por una sola rama.
//- Para borrar solo existe pop_min, que es lo único que necesita A*: el siguiente mínimo
//  siempre es el mínimo del primer cluster, así que nunca hay que buscar una clave arbitraria.

const LEAF_BITS: u32 = 8;

//Hash para claves u64: una multiplicación de 128 bits plegada (la misma idea de foldhash).
//Mezcla tanto los bits bajos (hashbrown elige el bucket con ellos) como los altos
//(hashbrown los usa en los bytes de control).
#[derive(Default)]
pub(crate) struct FoldHasher(u64);

impl Hasher for FoldHasher {
    fn write(&mut self, bytes: &[u8]) {
        //las claves son u64 y pasan por write_u64; esto solo existe porque el trait lo exige
        for &b in bytes {
            self.write_u64(b as u64);
        }
    }

    fn write_u64(&mut self, x: u64) {
        let product = ((self.0 ^ x) as u128).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.0 = (product as u64) ^ ((product >> 64) as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

//HashMap con claves u64 y el hash rápido; también lo usa la open del vEB para agrupar empates
pub(crate) type FastHashMap<V> = HashMap<u64, V, BuildHasherDefault<FoldHasher>>;

type ClusterMap = FastHashMap<Veb>;

//Universo de hasta 256 claves: un bit por clave
#[derive(Clone, Copy, Default)]
struct Leaf([u64; 4]);

impl Leaf {
    fn insert(&mut self, x: u64) -> bool {
        let word = &mut self.0[(x >> 6) as usize];
        let bit = 1u64 << (x & 63);
        let added = *word & bit == 0;
        *word |= bit;
        added
    }

    fn contains(&self, x: u64) -> bool {
        self.0[(x >> 6) as usize] & (1u64 << (x & 63)) != 0
    }

    fn min(&self) -> Option<u64> {
        for (i, &word) in self.0.iter().enumerate() {
            if word != 0 {
                return Some(((i as u64) << 6) | word.trailing_zeros() as u64);
            }
        }
        None
    }

    fn max(&self) -> Option<u64> {
        for (i, &word) in self.0.iter().enumerate().rev() {
            if word != 0 {
                return Some(((i as u64) << 6) | (63 - word.leading_zeros()) as u64);
            }
        }
        None
    }

    fn pop_min(&mut self) -> Option<u64> {
        for (i, word) in self.0.iter_mut().enumerate() {
            if *word != 0 {
                let min = ((i as u64) << 6) | word.trailing_zeros() as u64;
                //apaga el bit encendido más bajo
                *word &= *word - 1;
                return Some(min);
            }
        }
        None
    }

    fn is_empty(&self) -> bool {
        self.0 == [0; 4]
    }
}

//Nodo interno: nunca está vacío, porque los clusters vacíos se eliminan
struct Node {
    min: u64,
    max: u64,
    //una clave x se parte en high(x) = x >> lower_bits y low(x) = los lower_bits de abajo
    lower_bits: u32,
    upper_bits: u32,
    //qué clusters tienen elementos (universo de 2^upper_bits); None si no hay clusters
    summary: Option<Box<Veb>>,
    clusters: ClusterMap,
}

impl Node {
    fn insert(&mut self, mut x: u64) -> bool {
        if x == self.min {
            return false;
        }
        if x < self.min {
            //x pasa a ser el mínimo y el mínimo anterior baja a los clusters
            mem::swap(&mut x, &mut self.min);
        }
        if x > self.max {
            self.max = x;
        }
        let high = x >> self.lower_bits;
        let low = x & ((1u64 << self.lower_bits) - 1);
        match self.clusters.entry(high) {
            Entry::Occupied(mut cluster) => cluster.get_mut().insert(low),
            Entry::Vacant(slot) => {
                //cluster nuevo: guardarle low es O(1), y la recursión sigue solo por el summary
                slot.insert(Veb::singleton(self.lower_bits, low));
                match &mut self.summary {
                    Some(summary) => {
                        summary.insert(high);
                    }
                    None => self.summary = Some(Box::new(Veb::singleton(self.upper_bits, high))),
                }
                true
            }
        }
    }

    fn contains(&self, x: u64) -> bool {
        if x == self.min || x == self.max {
            return true;
        }
        let high = x >> self.lower_bits;
        let low = x & ((1u64 << self.lower_bits) - 1);
        self.clusters.get(&high).is_some_and(|cluster| cluster.contains(low))
    }

    //saca el mínimo; el bool indica si el nodo quedó vacío
    fn pop_min(&mut self) -> (u64, bool) {
        let old_min = self.min;
        let Some(summary) = self.summary.as_mut() else {
            //sin clusters el nodo solo tenía su mínimo
            return (old_min, true);
        };
        //el nuevo mínimo es el mínimo del primer cluster con elementos
        let high = summary.min();
        let cluster = self
            .clusters
            .get_mut(&high)
            .expect("el summary apunta a un cluster que no existe");
        let (low, cluster_empty) = cluster.pop_min();
        self.min = (high << self.lower_bits) | low;
        if cluster_empty {
            self.clusters.remove(&high);
            //high era el mínimo del summary, así que también sale con pop_min
            let (_, summary_empty) = summary.pop_min();
            if summary_empty {
                self.summary = None;
                self.max = self.min;
            }
        }
        (old_min, false)
    }
}

//Un vEB de universo 2^bits: hoja si bits <= LEAF_BITS, nodo en otro caso.
//Siempre tiene al menos un elemento: el árbol vacío se representa afuera con None.
enum Veb {
    Leaf(Leaf),
    Node(Node),
}

impl Veb {
    fn singleton(bits: u32, x: u64) -> Veb {
        if bits <= LEAF_BITS {
            let mut leaf = Leaf::default();
            leaf.insert(x);
            Veb::Leaf(leaf)
        } else {
            let lower_bits = bits / 2;
            Veb::Node(Node {
                min: x,
                max: x,
                lower_bits,
                upper_bits: bits - lower_bits,
                summary: None,
                clusters: ClusterMap::default(),
            })
        }
    }

    fn min(&self) -> u64 {
        match self {
            Veb::Leaf(leaf) => leaf.min().expect("hoja vacía"),
            Veb::Node(node) => node.min,
        }
    }

    fn max(&self) -> u64 {
        match self {
            Veb::Leaf(leaf) => leaf.max().expect("hoja vacía"),
            Veb::Node(node) => node.max,
        }
    }

    fn insert(&mut self, x: u64) -> bool {
        match self {
            Veb::Leaf(leaf) => leaf.insert(x),
            Veb::Node(node) => node.insert(x),
        }
    }

    fn contains(&self, x: u64) -> bool {
        match self {
            Veb::Leaf(leaf) => leaf.contains(x),
            Veb::Node(node) => node.contains(x),
        }
    }

    fn pop_min(&mut self) -> (u64, bool) {
        match self {
            Veb::Leaf(leaf) => {
                let min = leaf.pop_min().expect("hoja vacía");
                (min, leaf.is_empty())
            }
            Veb::Node(node) => node.pop_min(),
        }
    }

    fn heap_bytes(&self) -> usize {
        match self {
            //las hojas y los nodos van dentro del enum; lo que sí pide memoria aparte
            //son las tablas de clusters y el Box de cada summary
            Veb::Leaf(_) => 0,
            Veb::Node(node) => {
                hashmap_heap_bytes(&node.clusters)
                    + node.summary.as_ref().map_or(0, |summary| size_of::<Veb>() + summary.heap_bytes())
                    + node.clusters.values().map(Veb::heap_bytes).sum::<usize>()
            }
        }
    }

    //como heap_bytes, pero cada tabla cuenta solo sus entradas ocupadas (len en vez de capacidad)
    fn used_bytes(&self) -> usize {
        match self {
            Veb::Leaf(_) => 0,
            Veb::Node(node) => {
                node.clusters.len() * size_of::<(u64, Veb)>()
                    + node.summary.as_ref().map_or(0, |summary| size_of::<Veb>() + summary.used_bytes())
                    + node.clusters.values().map(Veb::used_bytes).sum::<usize>()
            }
        }
    }
}

//Conjunto de claves u64 con insert, min y pop_min en O(log log U)
pub struct VebTree {
    bits: u32,
    root: Option<Veb>,
    len: usize,
}

impl VebTree {
    //universo de 2^bits claves, con bits entre 1 y 64
    pub fn new(bits: u32) -> Self {
        assert!((1..=64).contains(&bits), "el universo debe tener entre 1 y 64 bits");
        Self { bits, root: None, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn min(&self) -> Option<u64> {
        self.root.as_ref().map(Veb::min)
    }

    pub fn max(&self) -> Option<u64> {
        self.root.as_ref().map(Veb::max)
    }

    pub fn contains(&self, x: u64) -> bool {
        self.root.as_ref().is_some_and(|root| root.contains(x))
    }

    //devuelve false si la clave ya estaba
    pub fn insert(&mut self, x: u64) -> bool {
        debug_assert!(self.bits == 64 || x >> self.bits == 0, "clave fuera del universo");
        let added = match &mut self.root {
            Some(root) => root.insert(x),
            None => {
                self.root = Some(Veb::singleton(self.bits, x));
                true
            }
        };
        if added {
            self.len += 1;
        }
        added
    }

    pub fn pop_min(&mut self) -> Option<u64> {
        let (min, now_empty) = self.root.as_mut()?.pop_min();
        if now_empty {
            self.root = None;
        }
        self.len -= 1;
        Some(min)
    }

    //memoria que el árbol pide al allocator (nodos y tablas de clusters), sin contar el struct
    pub fn heap_bytes(&self) -> usize {
        self.root.as_ref().map_or(0, Veb::heap_bytes)
    }

    //de heap_bytes, lo que tiene datos: entradas ocupadas en las tablas de clusters y los summaries
    pub fn used_bytes(&self) -> usize {
        self.root.as_ref().map_or(0, Veb::used_bytes)
    }
}

impl Default for VebTree {
    fn default() -> Self {
        Self::new(64)
    }
}
