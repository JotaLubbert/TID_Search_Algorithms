use std::collections::HashMap;
use crate::a_star::{close_size_in_bytes, Coords, SearchNode};
use crate::hasher::CoordBuild;

pub trait ClosedSet {
    fn new() -> Self;
    fn get(&self, c: Coords) -> Option<&SearchNode>;
    fn insert(&mut self, node: SearchNode);
    fn size_in_bytes(&self) -> usize;
    fn len(&self) -> usize;
}

impl ClosedSet for HashMap<Coords, SearchNode> {
    fn new() -> Self {HashMap::new()}
    fn get(&self, c: Coords) -> Option<&SearchNode> {HashMap::get(self, &c)}
    fn insert(&mut self, node: SearchNode) {HashMap::insert(self, node.coords, node);}
    fn len(&self) -> usize {HashMap::len(self)}
    fn size_in_bytes(&self) -> usize {close_size_in_bytes(self)}
}

impl ClosedSet for HashMap<Coords, SearchNode, CoordBuild> {
    fn new() -> Self {HashMap::with_hasher(CoordBuild::default())}
    fn get(&self, c: Coords) -> Option<&SearchNode> {HashMap::get(self, &c)}
    fn insert(&mut self, node: SearchNode) {HashMap::insert(self, node.coords, node);}
    fn len(&self) -> usize {HashMap::len(self)}
    fn size_in_bytes(&self) -> usize {close_size_in_bytes(self)}
}