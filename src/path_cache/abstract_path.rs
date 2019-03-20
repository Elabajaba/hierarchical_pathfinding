use super::path_segment::{PathSegment, PathSegment::*};
use crate::{
	generics::{a_star_search, Cost, Path},
	neighbors::Neighborhood,
	Point,
};

#[derive(Debug, Clone)]
pub struct AbstractPath<N: Neighborhood> {
	neighborhood: N,
	total_cost: Cost,
	path: Vec<PathSegment>,
	end: Point,
	current_index: (usize, usize),
}

#[allow(dead_code)]
impl<N: Neighborhood> AbstractPath<N> {
	/// Returns the total cost of this Path.
	/// This value is always known and requires no further calculations.
	pub fn cost(&self) -> Cost {
		self.total_cost
	}

	/// A variant of [`Iterator::next()`](#impl-Iterator) that can resolve unknown segments
	/// of the Path. Use this method instead of `next()` when
	/// [`config.cache_paths`](super::PathCacheConfig::cache_paths) is set to false.
	pub fn safe_next(&mut self, get_cost: impl Fn(Point) -> isize) -> Option<Point> {
		if self.current_index.0 >= self.path.len() {
			return None;
		}
		let mut current = &self.path[self.current_index.0];
		if let Unknown { start, end, .. } = *current {
			let path = a_star_search(
				|p| {
					self.neighborhood
						.get_all_neighbors(p)
						.map(|n| (n, get_cost(n) as usize))
				},
				|p| get_cost(p) >= 0,
				start,
				end,
				|p| self.neighborhood.heuristic(p, end),
			)
			.unwrap_or_else(|| {
				panic!(
					"Impossible Path marked as Possible: {:?} -> {:?}",
					start, end
				)
			});

			self.path[self.current_index.0] = Known(path);
			current = &self.path[self.current_index.0];

			self.current_index.1 = 1; // paths include start and end, but we are already at start
		}

		if let Known(path) = current {
			let ret = path[self.current_index.1];
			self.current_index.1 += 1;
			if self.current_index.1 >= path.len() {
				self.current_index.0 += 1;
				self.current_index.1 = 0;
			}
			Some(ret)
		} else {
			panic!("how.");
		}
	}

	pub(crate) fn new(neighborhood: N, start: Point) -> AbstractPath<N> {
		AbstractPath {
			neighborhood: neighborhood,
			total_cost: 0,
			path: vec![],
			end: start,
			current_index: (0, 1),
		}
	}

	pub(crate) fn from_known_path(neighborhood: N, path: Path<Point>) -> AbstractPath<N> {
		let end = path[path.len() - 1];
		AbstractPath {
			neighborhood: neighborhood,
			total_cost: path.cost,
			path: vec![Known(path)],
			end,
			current_index: (0, 1),
		}
	}

	pub(crate) fn from_node(neighborhood: N, node: Point) -> AbstractPath<N> {
		AbstractPath {
			neighborhood: neighborhood,
			total_cost: 0,
			path: vec![],
			end: node,
			current_index: (0, 1),
		}
	}

	pub(crate) fn add_path_segment(&mut self, path: PathSegment) -> &mut Self {
		assert!(self.end == path.start(), "Added disconnected PathSegment");
		self.total_cost += path.cost();
		self.end = path.end();
		self.path.push(path);
		self
	}

	pub(crate) fn add_path(&mut self, path: Path<Point>) -> &mut Self {
		self.total_cost += path.cost;
		self.end = path[path.len() - 1];
		self.path.push(Known(path));
		self
	}

	pub(crate) fn add_node(&mut self, node: Point, cost: Cost, len: usize) -> &mut Self {
		self.path.push(Unknown {
			start: self.end,
			end: node,
			cost,
			len,
		});
		self.total_cost += cost;
		self.end = node;
		self
	}
}

impl<N: Neighborhood> Iterator for AbstractPath<N> {
	type Item = Point;
	/// See [`Iterator::next`]
	/// 
	/// ## Panics
	/// Panics if a segment of the Path is not known because [`config.cache_paths`](super::PathCacheConfig::cache_paths)
	/// is set to false. Use [`safe_next`](AbstractPath::safe_next) in those cases.
	fn next(&mut self) -> Option<Point> {
		if self.current_index.0 >= self.path.len() {
			return None;
		}
		let current = &self.path[self.current_index.0];
		if let Unknown { .. } = *current {
			panic!(
				"Tried calling next() on a Path that is not fully known. Use safe_next instead."
			);
		}

		if let Known(path) = current {
			let ret = path[self.current_index.1];
			self.current_index.1 += 1;
			if self.current_index.1 >= path.len() {
				self.current_index.0 += 1;
				self.current_index.1 = 1;
			}
			Some(ret)
		} else {
			panic!("how.");
		}
	}
}
