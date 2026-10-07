use std::cmp::Reverse;
use std::collections::BinaryHeap;
use radix_heap;
use std::collections::hash_map::Entry;
use std::iter;
use std::mem::size_of;
use crate::a_star::{SearchNode, hashmap_heap_bytes};
use crate::radix_alt::RadixHeap;
use crate::veb::{FastHashMap, VebTree};

//Implementación base: BinaryHeap es un max-heap, así que con Reverse queda como min-heap por f
pub type BinaryHeapOpen = BinaryHeap<Reverse<SearchNode>>;
pub type RadixHeapOpen = radix_heap::RadixHeapMap<Reverse<u64>, SearchNode>;

//El vEB es un conjunto de claves, así que los nodos van aparte, agrupados por clave.
//La clave es f.to_bits(): exacta, y como f >= 0 sus bits mantienen el orden de f.
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
//- la clave es la misma del original, f.to_bits(): exacta, así que A* no necesita cambios y
//  expande los mismos nodos que el radix original; la diferencia de tiempo es solo de la estructura
#[derive(Default)]
pub struct RadixAltOpen {
    heap: RadixHeap<u32>,
    slots: Vec<SearchNode>,
    free_slots: Vec<u32>,
}

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
    //recorre los nodos sin importar el orden, se usa para la visualización
    fn nodes(&self) -> impl Iterator<Item = &SearchNode>;
}


impl OpenList for BinaryHeapOpen {
    const NAME: &'static str = "binary-heap";
    
    fn insert(&mut self, node: SearchNode) {
        self.push(Reverse(node));
    }

    fn pop_min(&mut self) -> Option<SearchNode> {
        self.pop().map(|Reverse(node)| node)
    }

    //size_of_val no sirve acá: solo mide el struct en el stack y el buffer del heap
    //queda fuera, que es justamente lo que crece durante la búsqueda.
    fn size_in_bytes(&self) -> usize {
        size_of::<Self>() + self.capacity() * size_of::<Reverse<SearchNode>>()
    }

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.iter().map(|Reverse(node)| node)
    }
}

impl OpenList for RadixHeapOpen{
    const NAME: &'static str = "radix-heap";

    fn insert(&mut self, node: SearchNode){
        let mut key: u64 = node.f.to_bits();
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

    fn nodes(&self) -> impl Iterator<Item = &SearchNode>{
        self.iter().map(|(Reverse(_key), node)| node)
    }
}

impl OpenList for VebOpen {
    const NAME: &'static str = "veb-tree";

    fn insert(&mut self, node: SearchNode) {
        let key = node.f.to_bits();
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

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.buckets.values().flat_map(|b| iter::once(&b.first).chain(b.rest.iter()))
    }
}

impl OpenList for RadixAltOpen {
    const NAME: &'static str = "radix-alt";

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
        //f >= 0, así que sus bits como u64 mantienen el orden de f. El ajuste a top cubre
        //el ruido de redondeo de f, igual que en el radix original
        let key = node.f.to_bits().max(self.heap.top());
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

    fn nodes(&self) -> impl Iterator<Item = &SearchNode> {
        self.heap.values().map(|&slot| &self.slots[slot as usize])
    }
}
