#![allow(unused)]
use std::io::Write;

use hashbrown::HashMap;
use slotmap::{DefaultKey, SecondaryMap, SlotMap, SparseSecondaryMap};

use crate::{utils::IterExt, NodeID, Point, PointMap};

use super::{Cost, Path};

// SOA layout
#[derive(Clone, Debug)]
struct PathStorage {
    paths: SlotMap<DefaultKey, Path3>,
    costs: SecondaryMap<DefaultKey, Cost>,
    starts: SparseSecondaryMap<DefaultKey, Point>,
    ends: SparseSecondaryMap<DefaultKey, Point>,
    start_costs: SecondaryMap<DefaultKey, Cost>,
    end_costs: SecondaryMap<DefaultKey, Cost>,
    // Since a path always only has 2 owners, we don't need to track them here. We track them in the pos_map at a level above here.
    // owners: HashMap<DefaultKey, Vec<NodeID>>,
}

impl PathStorage {
    pub fn new() -> Self {
        PathStorage {
            paths: SlotMap::new(),
            costs: SecondaryMap::new(),
            starts: SparseSecondaryMap::new(),
            ends: SparseSecondaryMap::new(),
            start_costs: SecondaryMap::new(),
            end_costs: SecondaryMap::new(),
            // owners: HashMap::new(),
        }
    }

    fn insert_new_path(&mut self, path: Path<Point>) -> DefaultKey {
        // let mut p = path.path.to_vec();
        // p.shrink_to_fit();
        let key = self.paths.insert(Path3::Path(path.path.to_vec()));
        match self.paths[key] {
            Path3::Path(_) => { // do nothing
            }
            Path3::CompressedPath(_) => {
                self.starts.insert(key, path[0]);
                self.ends.insert(key, path[path.len() - 1]);
            }
        }
        self.costs.insert(key, path.cost());

        key
    }

    fn delete_path(&mut self, key: DefaultKey) {
        self.paths.remove(key);
        self.costs.remove(key);
        self.starts.remove(key);
        self.ends.remove(key);
    }

    // TODO: IsReversed isn't necessary here, but it's easier to call this way.
    fn contains_path(&self, key: DefaultKey, path_end: &Point) -> Option<DefaultKey> {
        if self.end(key) == *path_end {
            return Some(key);
        }
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
        match &self.paths[key] {
            Path3::Path(path) => path[0],
            Path3::CompressedPath(_) => self.starts[key],
        }
    }

    fn end(&self, key: DefaultKey) -> Point {
        match &self.paths[key] {
            Path3::Path(path) => path[path.len() - 1],
            Path3::CompressedPath(_) => self.ends[key],
        }
    }

    // In order to reverse a path, need to know the start and end costs.
    fn reverse_path(&self, key: DefaultKey) -> Path2 {
        let mut path = self.path(key).clone();
        path.reverse();
        path
    }

    #[track_caller]
    fn reverse_cost(&self, key: DefaultKey) -> usize {
        // TODO: Fix this.
        self.cost(key)
        // unimplemented!()
    }

    fn delete_everything(&mut self) {
        use std::thread::sleep;
        use std::time::Duration;
        println!("dropping end_costs");
        self.end_costs = SecondaryMap::new();
        sleep(Duration::new(5, 0));
        println!("dropping start_costs");
        self.start_costs = SecondaryMap::new();
        sleep(Duration::new(5, 0));
        println!("dropping ends");
        self.ends = SparseSecondaryMap::new();
        sleep(Duration::new(5, 0));
        println!("dropping starts");
        self.starts = SparseSecondaryMap::new();
        sleep(Duration::new(5, 0));
        println!("dropping costs");
        self.costs = SecondaryMap::new();
        sleep(Duration::new(5, 0));
        println!("dropping paths");
        self.paths = SlotMap::new();
    }

    fn shrink(&mut self) {
        // self.paths.
    }

    fn print_points_in_paths(&self) {
        println!("capacity: {}", self.paths.capacity());
        let mut len = 0;
        let mut printed = false;
        for path in self.paths.values() {
            match path {
                Path3::Path(p) => {
                    len += p.len();
                    if !printed {
                        printed = true;
                        let cap = p.capacity();
                        use std::mem::size_of;

                        println!(
                            "pcap: {}, plen: {}, point size: {}",
                            cap,
                            p.len(),
                            size_of::<Point>(),
                        );
                    }
                }
                Path3::CompressedPath(_) => todo!(),
            }
        }

        println!("length: {}", len);
    }
}

#[derive(Clone, Debug)]
enum Path3 {
    Path(Path2),
    // #[cfg(feature = "compression")]
    CompressedPath(Vec<u8>),
}

type Path2 = Vec<Point>;
type IsReversed = bool;

// Thoughts: This is ugly. The point of the SOA rewrite was to be able to access paths/costs/etc in chunks for simd.
// TODO: Only create paths such that A < B, which will allow for speeding up and simplifiying adding and removing paths (fewer branches).
// TODO:
/// A convenient wrapper that associates points with defaultkeys in a path storage.
/// Has a thin layer of abstraction over a PathStorage to allow for transparent path deduplication.
#[derive(Clone, Debug)]
pub struct PathStorageWrapper {
    path_storage: PathStorage,
    /// Anything above this layer shouldn't need to know if a path is reversed or not.
    /// IsReversed can be stored here. If the stored path is Point A-> Point B, and we check Point A, then reversed is false.
    /// If we check Point B, then reversed is true.
    // TODO: This currently disallows batched adding of paths and their owners, we have to add new owners one by one for is_reversed to be correct.
    pos_map: PointMap<(Vec<(DefaultKey, IsReversed)>)>,
}

impl<'a> PathStorageWrapper {
    pub(crate) fn new() -> Self {
        PathStorageWrapper {
            path_storage: PathStorage::new(),
            pos_map: PointMap::new(),
        }
    }

    pub fn temp_print(&self) {
        self.path_storage.print_points_in_paths();
    }

    pub fn drop_path_storage(&mut self) {
        self.path_storage.delete_everything();
    }

    pub fn drop_pos_map(&mut self) {
        self.pos_map.clear();
        self.pos_map.shrink_to_fit();
    }

    pub fn shrink_paths(&mut self) {
        self.path_storage.shrink();
    }

    pub fn print_paths(&self) {
        use std::fs::File;

        let mut file = File::create("new-paths.txt").unwrap();

        for (pos, thing) in self.pos_map.iter() {
            // println!("pos: {:?}", pos);
            for (key, is_rev) in thing {
                let path = self.get_path(*key, *is_rev);
                let out = format!("{:?}\n", path);
                file.write(out.as_bytes());
                // println!("{:?}", path);
            }
            // println!("");
        }
    }

    // TODO: Path A<->B is only ever owned by A and B. This is overabstracted for that.
    // When we generate a path between A & B, do we need to care what way it was generated?
    // Potentially for scheduling path generation. For this, we don't care. Just pass in both owners and the path,
    // and we can shove them in.
    // Issue: Can't batch add owners, as we don't know if they're reversed or not.
    /// If a path with the same start and end points already exists in the `PathStorage`, then add a new owner to it.
    /// If a reversed version of a path exists (NewStart=ExistingEnd, NewEnd=ExistingStart),
    /// then add a new owner to it and set IsReversed to True in the pos_map.
    /// If the path doesn't exist, then add a new path to the PathStorage.
    pub(crate) fn insert(&mut self, path: Path<Point>) -> DefaultKey {
        let path_start = path[0];
        let path_end = path[path.len() - 1];
        // println!("path start: {:?}, path_end: {:?}", path_start, path_end);
        // TODO: Is there some way to refactor these first to if let Some(), since they only differ in
        // reversing the start and end.
        // TODO: Potential logic error in is_reversed here? Test to confirm.
        // TODO: If we rework path generation to only do A->B, and not B->A (eg. maybe only generate the A->B path where the coordinates increase?)
        // then we can remove one of these branches.
        if let Some(keys) = self.pos_map.get(&path_start) {
            // println!(
            //     "might exist?: path start: {:?}, path_end: {:?}",
            //     path_start, path_end
            // );
            // Path might already exist?
            for (key, is_rev) in keys {
                let end = if *is_rev { path_start } else { path_end };

                if let Some(key) = self.path_storage.contains_path(*key, &path_end) {
                    // println!(
                    //     "already exists: path start: {:?}, path_end: {:?}",
                    //     path_start, path_end
                    // );
                    return key;
                }
            }
        }
        // Path doesn't exist
        // println!(
        //     "doesn't exist: path start: {:?}, path_end: {:?}",
        //     path_start, path_end
        // );

        let path_key = self.path_storage.insert_new_path(path);
        let keys = self.pos_map.entry(path_start).or_default();
        keys.push((path_key, false));
        let rev_keys = self.pos_map.entry(path_end).or_default();
        rev_keys.push((path_key, true));
        return path_key;
    }

    pub(crate) fn remove_path(&mut self, start: Point, end: Point) {
        let keys = self
            .pos_map
            .get(&start)
            .expect("Failed to remove path. Path at position doesn't exist");
        // TODO: I can get all paths from pos_map for a position, but how do I get a single path from pos to end?
        let id = self.find_path(end, keys).expect(
            "Failed to remove path. Can't find a path containing given start and end points.",
        );

        self.path_storage.delete_path(id);
    }

    // Remove all paths for a chunk
    pub(crate) fn remove_all_paths_containing_node(&mut self, node: Point) {
        let mut to_remove = HashMap::new();

        let keys = self
            .pos_map
            .get(&node)
            .expect("Failed to remove path. No paths exist for given node.");

        for (key, is_rev) in keys {
            let other_pos = if *is_rev {
                self.path_storage.start(*key)
            } else {
                self.path_storage.end(*key)
            };

            let entry = to_remove.entry(other_pos).or_insert(Vec::new());
            entry.push(*key);
            self.path_storage.delete_path(*key);
        }

        self.pos_map.remove(&node);

        for (pos, keys) in to_remove {
            let entries = self.pos_map.get_mut(&pos).unwrap();
            entries.retain(|(a, _)| !keys.contains(a));
        }

        // to_remove
    }

    // TODO: This is hideous.
    fn find_path(&self, end: Point, keys: &[(DefaultKey, IsReversed)]) -> Option<DefaultKey> {
        for &(key, is_rev) in keys {
            if is_rev {
                if self.path_storage.starts.contains_key(key) {
                    return Some(key);
                }
            } else if self.path_storage.ends.contains_key(key) {
                return Some(key);
            }
        }

        None
    }

    pub(crate) fn get_path(&self, key: DefaultKey, is_rev: bool) -> Path2 {
        let path = if !is_rev {
            self.path_storage.path(key).clone()
        } else {
            self.path_storage.reverse_path(key)
        };

        path
    }

    pub(crate) fn get_edges(&self, pos: Point) -> &Vec<(DefaultKey, bool)> {
        &self.pos_map[&pos]
    }

    pub(crate) fn get_cost(&self, key: &DefaultKey, is_rev: &bool) -> usize {
        let cost = if !is_rev {
            self.path_storage.cost(*key)
        } else {
            self.path_storage.reverse_cost(*key)
        };

        cost
    }

    pub(crate) fn get_key(&self, start_pos: Point, end_pos: Point) -> (DefaultKey, bool) {
        // TODO: I can get all paths from pos_map for a position, but how do I get a single path from pos to end?
        let mut start_keys = self
            .pos_map
            .get(&start_pos)
            .expect("Failed to get path. Start position doesn't have any associated paths.");

        let end_keys = self
            .pos_map
            .get(&end_pos)
            .expect("Failed to get path. End position doesn't have any associated paths.");

        let temp = start_keys
            .iter()
            .filter(|(k, b)| end_keys.contains(&(*k, !b)))
            .to_vec();

        debug_assert_eq!(temp.len(), 1);
        *temp[0]
        // if temp.len() == 1 {
        //     *temp[0]
        // } else {
        //     panic!("get_path failed. Too many keys.");
        // }
    }

    pub(crate) fn get_end(&self, key: &DefaultKey, is_rev: bool) -> Point {
        if is_rev {
            self.path_storage.start(*key)
        } else {
            self.path_storage.end(*key)
        }
    }
}
