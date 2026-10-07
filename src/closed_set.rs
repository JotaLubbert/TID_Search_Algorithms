use crate::a_star::{Coords, SearchNode};

pub trait ClosedSet {
    fn new() -> Self;
    fn get(&self, c: Coords) -> Option<&SearchNode>;
    fn insert(&mut self, node: SearchNode);
    fn size_in_bytes(&self) -> usize;
    fn len(&self) -> usize;
}

impl ClosedSet for std::collections::HashMap<Coords, SearchNode> {
    fn new() -> Self { std::collections::HashMap::new() }
    fn get(&self, c: Coords) -> Option<&SearchNode> { std::collections::HashMap::get(self, &c) }
    fn insert(&mut self, node: SearchNode) { std::collections::HashMap::insert(self, node.coords, node); }
    fn size_in_bytes(&self) -> usize {crate::a_star::close_size_in_bytes(self)}
    fn len(&self) -> usize { std::collections::HashMap::len(self) }
}
