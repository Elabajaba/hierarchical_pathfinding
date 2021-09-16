use crate::{NodeID, Point};

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeID,
    pub pos: Point,
    pub walk_cost: usize,
}

impl Node {
    pub fn new(id: NodeID, pos: Point, walk_cost: usize) -> Node {
        Node { id, pos, walk_cost }
    }
}
