#![allow(unused)]
use hashbrown::HashMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

use crate::{NodeID, Point, PointMap};

use super::{Cost, Path};

// SOA layout
struct PathStorage {
    paths: SlotMap<DefaultKey, Path3>,
    costs: SecondaryMap<DefaultKey, Cost>,
    starts: SecondaryMap<DefaultKey, Point>,
    ends: SecondaryMap<DefaultKey, Point>,
    owners: HashMap<DefaultKey, Vec<NodeID>>,
}

impl PathStorage {
    pub fn new() -> Self {
        PathStorage {
            paths: SlotMap::new(),
            costs: SecondaryMap::new(),
            starts: SecondaryMap::new(),
            ends: SecondaryMap::new(),
            owners: HashMap::new(),
        }
    }

    fn insert_new_path(&mut self, path: Path<Point>, owner: NodeID) -> DefaultKey {
        let key = self.paths.insert(Path3::Path(path.path.to_vec()));
        self.costs.insert(key, path.cost());
        self.starts.insert(key, path[0]);
        self.ends.insert(key, path[path.len() - 1]);
        self.owners.insert(key, vec![owner]);

        key
    }

    fn delete_path(&mut self, key: DefaultKey) {
        self.paths.remove(key);
        self.costs.remove(key);
        self.starts.remove(key);
        self.ends.remove(key);
    }

    fn add_owner(&mut self, key: DefaultKey, owner: NodeID) {
        let owners = self
            .owners
            .get_mut(&key)
            .expect("Failed to remove owner from PathStorage, as the given key does not exist.");

        owners.push(owner);
    }

    fn remove_owner(&mut self, key: DefaultKey, owner: &NodeID) {
        let owners = self
            .owners
            .get_mut(&key)
            .expect("Failed to remove owner from PathStorage, as the given key does not exist.");
        owners.retain(|id| id != owner);
    }

    fn contains_path(
        &self,
        keys: &[(DefaultKey, IsReversed)],
        path_end: &Point,
    ) -> Option<(DefaultKey, IsReversed)> {
        None
    }

    fn path(&self, key: DefaultKey) -> &Path2 {
        match &self.paths[key] {
            Path3::Path(path) => path,
            Path3::CompressedPath(_) => todo!(),
        }
    }

    fn cost(&self, key: DefaultKey) -> Cost {
        self.costs[key]
    }

    fn start(&self, key: DefaultKey) -> Point {
        self.starts[key]
    }

    fn end(&self, key: DefaultKey) -> Point {
        self.ends[key]
    }
}

enum Path3 {
    Path(Path2),
    // #[cfg(feature = "compression")]
    CompressedPath(Vec<u8>),
}

type Path2 = Vec<Point>;
type IsReversed = bool;

// struct Path2 {
//     path_nodes: Vec<Point>,
// }

// Thoughts: This is ugly. The point of the SOA rewrite was to be able to access paths/costs/etc in chunks for simd.
/// A convenient wrapper that associates points with defaultkeys in a path storage.
/// Has a thin layer of abstraction over a PathStorage to allow for transparent path deduplication.
pub struct PathStorageWrapper {
    path_storage: PathStorage,
    /// Anything above this layer shouldn't need to know if a path is reversed or not.
    pos_map: PointMap<(Vec<(DefaultKey, IsReversed)>)>,
}

impl PathStorageWrapper {
    pub fn new() -> Self {
        PathStorageWrapper {
            path_storage: PathStorage::new(),
            pos_map: PointMap::new(),
        }
    }

    /// If a path with the same start and end points already exists in the `PathStorage`, then add a new owner to it.
    /// If a reversed version of a path exists (NewStart=ExistingEnd, NewEnd=ExistingStart),
    /// then add a new owner to it and set IsReversed to True in the pos_map.
    /// If the path doesn't exist, then add a new path to the PathStorage.
    pub fn insert(&mut self, owner: NodeID, path: Path<Point>) {
        let path_start = path[0];
        // TODO: Is there some way to refactor these first to if let Some(), since they only differ in
        // reversing the start and end.
        // TODO: Potential logic error in is_reversed here? Test to confirm.
        // TODO: If we rework path generation to only do A->B, and not B->A (eg. maybe only generate the A->B path where the coordinates increase?)
        // then we can remove one of these branches.
        if let Some(keys) = self.pos_map.get(&path_start) {
            // Path might already exist?
            if let Some((key, is_reversed)) =
                self.path_storage.contains_path(keys, &path[path.len() - 1])
            {
                // Path exists, do something
                self.path_storage.add_owner(key, owner);
                let keys = self.pos_map.entry(path_start).or_default();
                keys.push((key, is_reversed));
            }
        } else if let Some(keys) = self.pos_map.get(&path[path.len() - 1]) {
            // Path might exist in its reversed form?
            if let Some((key, is_reversed)) = self.path_storage.contains_path(keys, &path_start) {
                // Path exists, do something
                self.path_storage.add_owner(key, owner);
                let keys = self.pos_map.entry(path_start).or_default();
                keys.push((key, is_reversed));
            }
        } else {
            // Path doesn't exist
            let path_key = self.path_storage.insert_new_path(path, owner);
            let keys = self.pos_map.entry(path_start).or_default();
            // Default newly added paths to not be reversed.
            keys.push((path_key, false));
        }
    }

    pub fn remove_edge(&mut self, owner: NodeID) {}
}
