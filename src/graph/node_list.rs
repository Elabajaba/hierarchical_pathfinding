use super::{Node, NodeID, NodeIDMap, NodeIDSet};
use crate::{Point, PointMap};

#[derive(Clone, Debug)]
pub struct NodeList {
    nodes: Vec<Option<Node>>,
    pos_map: PointMap<NodeID>,
    next_id: usize,
}

impl NodeList {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            pos_map: PointMap::default(),
            next_id: 0,
        }
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.pos_map.len()
    }

    pub fn add_node(&mut self, pos: Point, walk_cost: usize) -> NodeID {
        while self.next_id < self.nodes.len() && self.nodes[self.next_id].is_some() {
            self.next_id += 1;
        }
        let raw_id = self.next_id;
        self.next_id += 1;
        let id = raw_id as NodeID;

        let node = Node::new(id, pos, walk_cost);
        if raw_id >= self.nodes.len() {
            self.nodes.push(Some(node));
        } else {
            self.nodes[raw_id] = Some(node);
        }
        self.pos_map.insert(pos, id);
        id
    }

    #[track_caller]
    pub fn remove_node(&mut self, id: NodeID) {
        let node = self.nodes[id as usize].take().unwrap();
        // for (other_id, _) in node.edges {
        //     self[other_id].edges.remove(&id);
        // }
        self.pos_map.remove(&node.pos);
        self.next_id = self.next_id.min(id as usize);
    }

    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator<Item = (NodeID, &Node)> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(id, opt)| opt.as_ref().map(|node| (id as NodeID, node)))
    }
    pub fn keys(&self) -> impl Iterator<Item = NodeID> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.is_some())
            .map(|(id, _)| id as NodeID)
    }

    #[allow(unused)]
    pub fn values(&self) -> impl Iterator<Item = &Node> + '_ {
        self.nodes.iter().filter_map(|opt| opt.as_ref())
    }

    pub fn id_at(&self, pos: Point) -> Option<NodeID> {
        self.pos_map.get(&pos).copied()
    }

    #[allow(unused)]
    pub fn absorb(&mut self, other: NodeList) -> NodeIDSet {
        let mut ret = NodeIDSet::default();
        let mut map = NodeIDMap::default();

        for node in other.nodes.iter().flatten() {
            let old = node.id;
            let new = self.add_node(node.pos, node.walk_cost);
            map.insert(old, new);
            ret.insert(new);
        }

        // for old_node in other.nodes.into_iter().flatten() {
        //     let mut new_node = &mut self[map[&old_node.id]];
        //     new_node.edges = old_node
        //         .edges
        //         .into_iter()
        //         .map(|(other_id, path)| (map[&other_id], path))
        //         .collect();
        // }

        ret
    }
}

use std::ops::{Index, IndexMut};
impl Index<NodeID> for NodeList {
    type Output = Node;
    #[track_caller]
    fn index(&self, index: NodeID) -> &Node {
        self.nodes[index as usize].as_ref().unwrap()
    }
}
impl IndexMut<NodeID> for NodeList {
    #[track_caller]
    fn index_mut(&mut self, index: NodeID) -> &mut Node {
        self.nodes[index as usize].as_mut().unwrap()
    }
}

// #[test]
// fn absorb() {
//     let mut nodes = NodeList::new();
//     nodes.add_node((0, 0), 0);
//     nodes.add_node((1, 1), 1);
//     nodes.add_node((2, 2), 2);
//     nodes.add_edge(
//         0,
//         1,
//         PathSegment::new(super::Path::from_slice(&[], 0), true),
//     );
//     nodes.add_edge(
//         2,
//         0,
//         PathSegment::new(super::Path::from_slice(&[], 2), true),
//     );

//     let mut new_nodes = NodeList::new();
//     new_nodes.add_node((10, 10), 10);
//     new_nodes.add_node((11, 11), 11);
//     new_nodes.add_edge(
//         0,
//         1,
//         PathSegment::new(super::Path::from_slice(&[], 10), true),
//     );

//     nodes.absorb(new_nodes);

//     assert_eq!(nodes.nodes.len(), 5);
//     assert_eq!(nodes.nodes[3].as_ref().unwrap().pos, (10, 10));
//     assert_eq!(nodes.nodes[4].as_ref().unwrap().pos, (11, 11));
//     assert_eq!(nodes.nodes[3].as_ref().unwrap().edges[&4].cost(), 10);
// }
