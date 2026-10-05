use std::cmp::Reverse;
use std::collections::BinaryHeap;
use radix_heap;
use std::mem::size_of;

//Implementación base: BinaryHeap es un max-heap, así que con Reverse queda como min-heap por f
pub type BinaryHeapOpen = BinaryHeap<Reverse<SearchNode>>;
pub type RadixHeapOpen = radix_heap::RadixHeapMap<Reverse<u64>, SearchNode>;

use crate::a_star::SearchNode;

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
