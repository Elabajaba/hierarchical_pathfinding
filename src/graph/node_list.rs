use slotmap::{DefaultKey, SlotMap};

use super::{Node, NodeID, NodeIDMap, NodeIDSet};
use crate::{path::PathSegment, Point, PointMap};

#[derive(Clone, Debug)]
pub struct NodeList {
    nodes: SlotMap<DefaultKey, Node>,
    pos_map: PointMap<DefaultKey>,
    // pos_map: SecondaryMap<DefaultKey, (usize, usize)>,
}

impl NodeList {
    // TODO: Size hint?
    pub fn new() -> Self {
        let nodes = SlotMap::new();
        Self {
            pos_map: PointMap::with_capacity(nodes.capacity()),
            nodes,
            // pos_map: SecondaryMap::with_capacity(nodes.capacity()),
        }
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    // If it exists, update it, otherwise add a new node.
    pub fn add_node(&mut self, pos: Point, walk_cost: usize) -> NodeID {
        let node = Node::new(pos, walk_cost);

        // if raw_id >= self.nodes.len() {
        //     self.nodes.push(Some(node));
        // } else {
        //     self.nodes[raw_id] = Some(node);
        // }

        // TODO: Do we need to check if a node exists before inserting a new one?

        let id = self.nodes.insert(node);
        self.pos_map.insert(pos, id);
        id
    }

    pub fn add_edge(&mut self, src: NodeID, target: NodeID, path: PathSegment) {
        let src_cost = self[src].walk_cost;

        let target_node = &mut self[target];

        let target_cost = target_node.walk_cost;

        let other_path = path.reversed(src_cost, target_cost);
        target_node.edges.insert(src, other_path);

        let src_node = &mut self[src];
        src_node.edges.insert(target, path);
    }

    #[track_caller]
    pub fn remove_node(&mut self, id: NodeID) {
        let node = self.nodes.remove(id).unwrap();
        // let node = self.nodes[id].take();
        for (other_id, _) in node.edges {
            self[other_id].edges.remove(&id);
        }
        self.pos_map.remove(&node.pos);
    }

    // #[allow(unused)]
    // pub fn iter(&self) -> impl Iterator<Item = (NodeID, &Node)> + '_ {
    //     self.nodes
    //         .iter()
    //         .enumerate()
    //         .filter_map(|(id, opt)| opt.as_ref().map(|node| (id as NodeID, node)))
    // }
    pub fn keys(&self) -> impl Iterator<Item = NodeID> + '_ {
        self.nodes.keys()
    }

    // #[allow(unused)]
    // pub fn values(&self) -> impl Iterator<Item = &Node> + '_ {
    //     self.nodes.iter().filter_map(|opt| opt.as_ref())
    // }

    pub fn id_at(&self, pos: Point) -> Option<NodeID> {
        self.pos_map.get(&pos).copied()
    }

    #[allow(unused)]
    pub fn absorb(&mut self, other: NodeList) -> NodeIDSet {
        // let mut ret = NodeIDSet::default();
        // let mut map = NodeIDMap::default();

        // // for node in other.nodes.iter().flatten() {
        // for (old, node) in other.nodes.iter() {
        //     let new = self.add_node(node.pos, node.walk_cost);
        //     map.insert(old, new);
        //     ret.insert(new);
        // }

        // for old_node in other.nodes.into_iter().flatten() {
        //     let mut new_node = &mut self[map[&old_node.id]];
        //     new_node.edges = old_node
        //         .edges
        //         .into_iter()
        //         .map(|(other_id, path)| (map[&other_id], path))
        //         .collect();
        // }
        // ret

        // We want to absorb the nodes from other into self.
        // TODO: Would it just be faster to move the old nodes into self, then recalculate edges?
        let mut ret = NodeIDSet::default();
        let mut old_to_new = NodeIDMap::default();

        for (old, node) in other.nodes.iter() {
            let new = self.add_node(node.pos, node.walk_cost);
            old_to_new.insert(old, new);
            ret.insert(new);
        }

        for (old, node) in other.nodes {
            let mut new_node = &mut self[old_to_new[&old]];

            new_node.edges = node
                .edges
                .into_iter()
                .map(|(other_id, path)| (old_to_new[&other_id], path))
                .collect();
        }

        ret
    }
}

use std::ops::{Index, IndexMut};
impl Index<NodeID> for NodeList {
    type Output = Node;
    #[track_caller]
    fn index(&self, index: NodeID) -> &Node {
        &self.nodes[index]
    }
}
impl IndexMut<NodeID> for NodeList {
    #[track_caller]
    fn index_mut(&mut self, index: NodeID) -> &mut Node {
        &mut self.nodes[index]
    }
}

#[test]
fn absorb() {
    let mut nodes = NodeList::new();
    let node_1_idx = nodes.add_node((0, 0), 0);
    let node_2_idx = nodes.add_node((1, 1), 1);
    let node_3_idx = nodes.add_node((2, 2), 2);
    nodes.add_edge(
        node_1_idx,
        node_2_idx,
        PathSegment::new(super::Path::from_slice(&[], 0), true),
    );
    nodes.add_edge(
        node_3_idx,
        node_1_idx,
        PathSegment::new(super::Path::from_slice(&[], 2), true),
    );

    let mut new_nodes = NodeList::new();
    let new_idx_1 = new_nodes.add_node((10, 10), 10);
    let new_idx_2 = new_nodes.add_node((11, 11), 11);
    new_nodes.add_edge(
        new_idx_1,
        new_idx_2,
        PathSegment::new(super::Path::from_slice(&[], 10), true),
    );

    nodes.absorb(new_nodes);

    

    assert_eq!(nodes.nodes.len(), 5);
    assert_eq!(nodes.nodes[nodes.id_at((10, 10)).unwrap()].pos, (10, 10));
    assert_eq!(nodes.nodes[nodes.id_at((11, 11)).unwrap()].pos, (11, 11));
    assert_eq!(nodes.nodes[nodes.id_at((10, 10)).unwrap()].edges[&nodes.id_at((11, 11)).unwrap()].cost(), 10);
}
