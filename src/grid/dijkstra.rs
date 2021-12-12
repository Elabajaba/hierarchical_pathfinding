// use rayon::prelude::*;

use super::{Element, Path};
use crate::{neighbors::Neighborhood, Point, PointMap, PointSet};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

// a Macro to log::trace the time since $timer, and restart $timer
#[cfg(feature = "log")]
macro_rules! re_trace {
    ($msg: literal, $timer: ident) => {
        // let now = std::time::Instant::now();
        // log::trace!(concat!("time to ", $msg, ": {:?}"), now - $timer);
        // #[allow(unused)]
        // let $timer = now;
    };
}
#[cfg(not(feature = "log"))]
macro_rules! re_trace {
    // does nothing without log feature
    ($msg: literal, $timer: ident) => {};
}

pub fn dijkstra_search<N: Neighborhood>(
    neighborhood: &N,
    mut valid: impl FnMut(Point) -> bool,
    mut get_cost: impl FnMut(Point) -> isize,
    start: Point,
    goals: &[Point],
    only_closest_goal: bool,
    size_hint: usize,
) -> PointMap<Path<Point>> {
    #[cfg(feature = "log")]
    let (outer_timer, timer) = (std::time::Instant::now(), std::time::Instant::now());

    if get_cost(start) < 0 {
        return PointMap::default();
    }
    let mut visited = PointMap::with_capacity(size_hint);
    let mut next = BinaryHeap::with_capacity(size_hint / 2);
    // let mut next = PriorityQueue::with_capacity(size_hint / 2);
    // let mut next = Vec::with_capacity(size_hint / 2);

    next.push(Element(start, 0));
    visited.insert(start, (0, start));

    let mut remaining_goals: PointSet = goals.iter().copied().collect();

    let mut goal_costs = PointMap::with_capacity(goals.len());

    let mut all_neighbors = vec![];

    re_trace!("dijkstra setup", timer);

    use std::time::{Duration, Instant};
    // let mut timers = [Duration::default(); 10];

    while let Some(Element(current_id, current_cost)) = next.pop() {
        // // let timer = Instant::now();
        match current_cost.cmp(&visited[&current_id].0) {
            Ordering::Greater => continue,
            Ordering::Equal => {}
            Ordering::Less => panic!("Binary Heap failed"),
        }
        // timers[0] += Instant::now() - timer;
        // // let timer = Instant::now();

        if remaining_goals.remove(&current_id) {
            goal_costs.insert(current_id, current_cost);
            if only_closest_goal || remaining_goals.is_empty() {
                break;
            }
        }

        // timers[1] += Instant::now() - timer;
        // // let timer = Instant::now();

        let delta_cost = get_cost(current_id);
        if delta_cost < 0 {
            continue;
        }
        let other_cost = current_cost + delta_cost as usize;

        // timers[2] += Instant::now() - timer;
        // // let timer = Instant::now();

        all_neighbors.clear();
        neighborhood.get_all_neighbors(current_id, &mut all_neighbors);

        // timers[3] += Instant::now() - timer;
        // let timer = Instant::now();

        for other_id in all_neighbors.iter() {
        // all_neighbors.iter().for_each(|other_id| {
            if !valid(*other_id) {
                // continue;
            } else if get_cost(*other_id) < 0 && !remaining_goals.contains(&other_id) {
                // continue;
            } else {
                let mut needs_visit = true;
                if let Some((prev_cost, prev_id)) = visited.get_mut(&other_id) {
                    if *prev_cost > other_cost {
                        *prev_cost = other_cost;
                        *prev_id = current_id;
                    } else {
                        needs_visit = false;
                    }
                } else {
                    visited.insert(*other_id, (other_cost, current_id));
                }

                if needs_visit {
                    next.push(Element(*other_id, other_cost));
                }
            }
        }

        // next.sort_unstable();

        // timers[4] += Instant::now() - timer;
    }

    re_trace!("dijkstra 1st loop", timer);

    // let mut goal_data = PointMap::with_capacity_and_hasher(goal_costs.len(), Default::default());

    let mut goal_data = PointMap::with_capacity(goal_costs.len());

    // for (&goal, &cost) in goal_costs.iter() {
        goal_costs.iter().for_each(|(&goal, &cost)| {
        // let timer = Instant::now();

        let steps = {
            let mut steps = vec![];
            let mut current = goal;

            while current != start {
                steps.push(current);
                let (_, prev) = visited[&current];
                current = prev;
            }
            steps.push(start);
            steps.reverse();
            steps
        };

        // timers[5] += Instant::now() - timer;
        // let timer = Instant::now();
        goal_data.insert(goal, Path::new(steps, cost));

        // timers[6] += Instant::now() - timer;
    });

    // for (idx, durations) in timers.iter().enumerate() {
    //     println!("idx: {}, duration: {:?}", idx, durations);
    // }

    re_trace!("dijkstra 2nd loop", timer);

    re_trace!("dijkstra total", outer_timer);

    goal_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        use crate::prelude::*;

        // create and initialize Grid
        // 0 = empty, 1 = swamp, 2 = wall
        let grid = [
            [0, 2, 0, 0, 0],
            [0, 2, 2, 2, 2],
            [0, 1, 0, 0, 0],
            [0, 1, 0, 2, 0],
            [0, 0, 0, 2, 0],
        ];
        let (width, height) = (grid.len(), grid[0].len());

        let neighborhood = ManhattanNeighborhood::new(width, height);

        const COST_MAP: [isize; 3] = [1, 10, -1];

        fn cost_fn(grid: &[[usize; 5]; 5]) -> impl '_ + FnMut(Point) -> isize {
            move |(x, y)| COST_MAP[grid[y][x]]
        }

        let start = (0, 0);
        let goals = [(4, 4), (2, 0)];

        let paths = dijkstra_search(
            &neighborhood,
            |_| true,
            cost_fn(&grid),
            start,
            &goals,
            false,
            40,
        );

        // (4, 4) is reachable
        assert!(paths.contains_key(&goals[0]));

        // (2, 0) is not reachable
        assert!(!paths.contains_key(&goals[1]));
    }
}
