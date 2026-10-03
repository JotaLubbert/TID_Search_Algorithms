use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::mem::size_of;

use crate::a_star::SearchNode;

//Interfaz de la open: A* solo necesita insertar y sacar el nodo de menor f.
//Como A* deja duplicados en la open y descarta los obsoletos al sacarlos,
//no hace falta poder actualizar la prioridad de un nodo que ya está adentro.
//Default permite que A* cree la open vacía sin saber qué estructura es.
pub trait OpenList: Default {
    fn insert(&mut self, node: SearchNode);
    fn pop_min(&mut self) -> Option<SearchNode>;
    //memoria que ocupa la estructura, contando su buffer en el heap
    fn size_in_bytes(&self) -> usize;
    //recorre los nodos sin importar el orden, se usa para la visualización
    fn nodes(&self) -> impl Iterator<Item = &SearchNode>;
}

//Implementación base: BinaryHeap es un max-heap, así que con Reverse queda como min-heap por f
pub type BinaryHeapOpen = BinaryHeap<Reverse<SearchNode>>;

impl OpenList for BinaryHeapOpen {
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
