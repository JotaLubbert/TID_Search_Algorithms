use std::cmp::Reverse;
use std::collections::BinaryHeap;
use radix_heap;
use std::collections::hash_map::Entry;
use std::iter;
use std::mem::size_of;
use crate::a_star::{SearchNode, hashmap_heap_bytes};
use crate::radix_alt::RadixHeap;
use crate::veb::{FastHashMap, VebTree};
use quickheap::{Avx2, ConfigurableSimdQuickHeap, Scalar, SimdElem};

//2^18: cada unidad de f queda partida en 262.144 segmentos. Con f < 16.384 la clave cabe en 32 bits,
//y QuickHeap lo necesita. Los laberintos llegan a f ~7.583 (con 2^20 el límite era 4.096 y no alcanzaba)
const ROUND_SCALE: f64 = (1u64 << 18) as f64;

//Clave de f que usan todas las estructuras: f amplificada por 2^18 y redondeada al entero más cercano,
//o sea f medida en segmentos de ~0,0000038 (8,6568… -> 2.269.342). Borra el ruido de las sumas de g
//(dos caminos con los mismos pasos en distinto orden dan f distintas en el último decimal) y así
//empatan los nodos con la misma f. Con la heurística octil es exacta: toda f es P + Q·√2 con enteros,
//y hasta f ~7.600 dos f distintas se separan por ~0,00015 o más, unas 39 veces el segmento. Con la euclidiana podría
//juntar f distintas que estén a menos de un segmento.
//Todas desempatan igual: entre claves iguales sale primero el último insertado (LIFO).
fn open_key(f: f64) -> u64 {
    (f * ROUND_SCALE).round() as u64
}

//Implementación base: BinaryHeap es un max-heap, así que con Reverse queda como min-heap.
//La entrada es (clave, Reverse(seq), nodo): seq es el número de inserción, único, así que entre
//claves iguales sale el de mayor seq (el último insertado) y el nodo nunca decide.
#[derive(Default)]
pub struct BinaryHeapOpen {
    heap: BinaryHeap<Reverse<BinaryEntry>>,
    seq: u32,
}
type BinaryEntry = (u64, Reverse<u32>, SearchNode);
pub type RadixHeapOpen = radix_heap::RadixHeapMap<Reverse<u64>, SearchNode>;

//El vEB es un conjunto de claves, así que los nodos van aparte, agrupados por clave.
//La clave es open_key(f); los empates ya son una pila (LIFO) porque rest hace push y pop.
#[derive(Default)]
pub struct VebOpen {
    tree: VebTree,
    buckets: FastHashMap<VebBucket>,
}

//nodos con la misma f; casi todas las f son únicas, así que el primero va sin Vec
//y la mayoría de las claves nunca pide memoria aparte
struct VebBucket {
    first: SearchNode,
    rest: Vec<SearchNode>,
}

//Variante optimizada del radix heap, para contrastar con el original (que no se toca):
//- usa el RadixHeap de radix_alt.rs, que encuentra el siguiente bucket con una máscara
//- el heap guarda índices u32 y los nodos viven en slots, que reutiliza los espacios liberados
//- la clave es open_key(f), igual que en las demás estructuras
#[derive(Default)]
pub struct RadixAltRoundOpen {
    heap: RadixHeap<u32>,
    slots: Vec<SearchNode>,
    free_slots: Vec<u32>,
}

//SimdQuickHeap (vendor/quickheap): parte la cola en capas con pivotes al azar, como un quickselect,
//usando SIMD (AVX2), y saca el mínimo de una última capa chica y ordenada.
//Solo guarda enteros, así que cada entrada es un u64 con la clave open_key(f) en los 32 bits altos y
//u32::MAX - seq en los 32 bajos (los nodos viven aparte, como en RadixAltRound):
//- la clave cabe en 32 bits hasta f < 16.384, y el insert lo verifica
//- seq es el número de inserción en esta búsqueda: no hay dos entradas iguales, y como se guarda
//  invertido, entre claves iguales sale primero el último insertado (LIFO), aunque los pivotes se
//  elijan al azar
//- slot_of[seq] dice en qué slot de slots quedó ese nodo (los slots se reutilizan con free_slots,
//  pero seq no, así que el orden de salida no depende de qué slot toque)
//S elige cómo compara la cola: Avx2 (4 claves por instrucción) o Scalar (el mismo algoritmo de a una
//clave, agregado en vendor/quickheap). Así la diferencia entre las dos es solo el SIMD.
pub struct QuickHeapBase<S: QuickBackend> {
    heap: ConfigurableSimdQuickHeap<u64, S>,
    slots: Vec<SearchNode>,
    free_slots: Vec<u32>,
    slot_of: Vec<u32>,
}
pub type QuickHeapOpen = QuickHeapBase<Avx2>;
pub type QuickHeapScalarOpen = QuickHeapBase<Scalar>;

//un backend de la cola y la carpeta donde van sus resultados
pub trait QuickBackend: SimdElem<u64> {
    const NAME: &'static str;
}
impl QuickBackend for Avx2 {
    const NAME: &'static str = "quick-heap";
}
impl QuickBackend for Scalar {
    const NAME: &'static str = "quick-heap-scalar";
}

//a mano y no con derive: derive le exigiría Default al backend, que es solo una marca de tipo
impl<S: QuickBackend> Default for QuickHeapBase<S> {
    fn default() -> Self {
        Self { heap: ConfigurableSimdQuickHeap::default(), slots: Vec::new(), free_slots: Vec::new(), slot_of: Vec::new() }
    }
}

const SEQ_MASK: u64 = u32::MAX as u64;

//Interfaz de la open: A* solo necesita insertar y sacar el nodo de menor f.
//Como A* deja duplicados en la open y descarta los obsoletos al sacarlos,
//no hace falta poder actualizar la prioridad de un nodo que ya está adentro.
//Default permite que A* cree la open vacía sin saber qué estructura es.
pub trait OpenList: Default {
    const NAME: &'static str;
    fn insert(&mut self, node: SearchNode);
    fn pop_min(&mut self) -> Option<SearchNode>;
    //memoria que ocupa la estructura, contando su buffer en el heap
    fn size_in_bytes(&self) -> usize;
    //de lo reservado, los bytes que tienen datos ahora: cuántos elementos hay por lo que ocupa cada uno
    fn used_bytes(&self) -> usize;
    //recorre los nodos sin importar el orden, se usa para la visualización
    fn nodes(&self) -> impl Iterator<Item = &SearchNode>;
}


impl OpenList for BinaryHeapOpen {
    const NAME: &'static str = "binary-heap";

    fn insert(&mut self, node: SearchNode) {
        let entry = (open_key(node.f), Reverse(self.seq), node);
        //assert y no wrapping: con seq repetido el desempate dejaría de ser LIFO sin avisar
        self.seq = self.seq.checked_add(1).expect("más de u32::MAX inserciones en una búsqueda");
        self.heap.push(Reverse(entry));
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        self.heap.pop().map(|Reverse((_, _, node))| node)
    }

    //size_of_val no sirve acá: solo mide el struct en el stack y el buffer del heap
    //queda fuera, que es justamente lo que crece durante la búsqueda.
    fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + self.heap.capacity() * size_of::<Reverse<BinaryEntry>>()
    }

    fn used_bytes(&self) -> usize {
        self.heap.len() * size_of::<Reverse<BinaryEntry>>()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.heap.iter().map(|Reverse((_, _, node))| node)
    }
}

impl OpenList for RadixHeapOpen{
    const NAME: &'static str = "radix-heap";

    fn insert(&mut self, node: SearchNode){
        let mut key: u64 = open_key(node.f);
        if let Some(Reverse(top)) = self.top(){
            key = key.max(top);
        }
        self.push(Reverse(key), node);
    }
    
    fn pop_min(&mut self) -> Option<SearchNode>{
        match self.pop(){
            Some(res)=>{Some(res.1)},
            None =>{None},
        }
    }
    
    fn size_in_bytes(&self) -> usize{
        size_of::<Self>() + self.heap_bytes()
    }

    //cada entrada del radix heap del crate es la clave junto al nodo completo
    fn used_bytes(&self) -> usize{
        self.len() * size_of::<(Reverse<u64>, SearchNode)>()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode>{
        self.iter().map(|(Reverse(_key), node)| node)
    }
}

impl OpenList for VebOpen {
    const NAME: &'static str = "veb-tree";

    fn insert(&mut self, node: SearchNode) {
        let key = open_key(node.f);
        match self.buckets.entry(key) {
            Entry::Occupied(mut bucket) => bucket.get_mut().rest.push(node),
            Entry::Vacant(slot) => {
                //clave nueva: entra al árbol; si ya existía, el árbol no cambia
                slot.insert(VebBucket { first: node, rest: Vec::new() });
                self.tree.insert(key);
            }
        }
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        let key = self.tree.min()?;
        let bucket = self.buckets.get_mut(&key).expect("el árbol tiene una clave sin nodos");
        if let Some(node) = bucket.rest.pop() {
            return Some(node);
        }
        //era el último nodo con esta f: la clave sale del árbol
        let node = bucket.first;
        self.buckets.remove(&key);
        self.tree.pop_min();
        Some(node)
    }

    fn size_in_bytes(&self) -> usize {
        size_of::<Self>()
            + self.tree.heap_bytes()
            + hashmap_heap_bytes(&self.buckets)
            + self.buckets.values().map(|b| b.rest.capacity() * size_of::<SearchNode>()).sum::<usize>()
    }

    //cada f distinta ocupa una entrada en buckets y su clave en el árbol; cada empate, un nodo en su Vec
    fn used_bytes(&self) -> usize {
        self.tree.used_bytes()
            + self.buckets.len() * size_of::<(u64, VebBucket)>()
            + self.buckets.values().map(|b| b.rest.len() * size_of::<SearchNode>()).sum::<usize>()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.buckets.values().flat_map(|b| iter::once(&b.first).chain(b.rest.iter()))
    }
}

impl OpenList for RadixAltRoundOpen {
    const NAME: &'static str = "radix-alt-round";

    fn insert(&mut self, node: SearchNode) {
        let slot = match self.free_slots.pop() {
            Some(slot) => {
                self.slots[slot as usize] = node;
                slot
            }
            None => {
                self.slots.push(node);
                (self.slots.len() - 1) as u32
            }
        };
        //el ajuste a top cubre el ruido de redondeo de f, igual que en el radix original
        let key = open_key(node.f).max(self.heap.top());
        self.heap.push(key, slot);
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        let (_, slot) = self.heap.pop()?;
        self.free_slots.push(slot);
        Some(self.slots[slot as usize])
    }

    fn size_in_bytes(&self) -> usize {
        size_of::<Self>()
            + self.heap.heap_bytes()
            + self.slots.capacity() * size_of::<SearchNode>()
            + self.free_slots.capacity() * size_of::<u32>()
    }

    //cada nodo vivo ocupa su entrada en el heap (clave e índice) y su lugar en slots
    fn used_bytes(&self) -> usize {
        self.heap.len() * (size_of::<(u64, u32)>() + size_of::<SearchNode>())
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.heap.values().map(|&slot| &self.slots[slot as usize])
    }
}

impl<S: QuickBackend> OpenList for QuickHeapBase<S>{
    const NAME: &'static str = S::NAME;
    fn insert(&mut self, node: SearchNode) {
        let slot = match self.free_slots.pop() {
            Some(slot) => {
                self.slots[slot as usize] = node;
                slot
            }
            None => {
                self.slots.push(node);
                (self.slots.len() - 1) as u32
            }
        };
        let key = open_key(node.f);
        //assert y no debug_assert: si la f no cupiera, el desplazamiento la cortaría sin avisar
        //y la entrada quedaría con otra clave. Estricto: con key = u32::MAX y seq = 0 la entrada
        //sería u64::MAX, el valor que el quickheap usa como relleno
        assert!(key < SEQ_MASK, "f = {} no cabe en los 32 bits de la clave", node.f);
        //slot_of tiene una entrada por inserción, así que su largo es el seq de este nodo
        let seq = u32::try_from(self.slot_of.len()).expect("más de u32::MAX inserciones en una búsqueda");
        self.slot_of.push(slot);
        //seq invertido: la entrada menor es la de mayor seq, o sea el último insertado
        self.heap.push((key << 32) | (u32::MAX - seq) as u64);
    }
    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.heap.iter().map(|&entry| &self.slots[self.slot_of[seq_of(entry)] as usize])
    }
    fn pop_min(&mut self) -> Option<SearchNode> {
        let slot = self.slot_of[seq_of(self.heap.pop()?)];
        self.free_slots.push(slot);
        Some(self.slots[slot as usize])
    }
    fn size_in_bytes(&self) -> usize {
        size_of::<Self>()
            + self.heap.heap_bytes()
            + self.slots.capacity() * size_of::<SearchNode>()
            + self.free_slots.capacity() * size_of::<u32>()
            + self.slot_of.capacity() * size_of::<u32>()
    }
    //cada nodo vivo ocupa su entrada u64 en la cola y su lugar en slots; slot_of crece con cada inserción
    fn used_bytes(&self) -> usize {
        (self.slots.len() - self.free_slots.len()) * (size_of::<u64>() + size_of::<SearchNode>())
            + self.slot_of.len() * size_of::<u32>()
    }

}

//número de inserción guardado (invertido) en los 32 bits bajos de una entrada
fn seq_of(entry: u64) -> usize {
    (u32::MAX as u64 - (entry & SEQ_MASK)) as usize
}
