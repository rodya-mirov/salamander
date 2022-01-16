use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use crate::components::*;
use crate::map::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
struct TilePriority(i32, WorldPos);

impl PartialOrd for TilePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TilePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .cmp(&other.0)
            .then(self.1.x.cmp(&other.1.x))
            .then(self.1.y.cmp(&other.1.y))
    }
}

pub fn distance_dijkstra_map<
    'a,
    T: Iterator<Item = &'a WorldPos>,
    IsBlocked: Fn(WorldPos) -> bool,
>(
    map: &Map,
    destinations: T,
    is_blocked: IsBlocked,
) -> HashMap<WorldPos, i32> {
    let start_time = std::time::Instant::now();

    let mut distances = HashMap::new(); // map WorldPos -> distance to player
    let mut to_process = BinaryHeap::new(); // newly adjacent tiles to consider

    for wp in destinations {
        to_process.push(TilePriority(0, *wp));
    }

    while let Some(TilePriority(priority, wp)) = to_process.pop() {
        let existing_priority = distances.get(&wp).copied().unwrap_or(i32::MAX);
        if priority < existing_priority {
            distances.insert(wp, priority);
            for tile in map.adjacent(wp) {
                if !is_blocked(tile) {
                    to_process.push(TilePriority(priority + 1, tile));
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_millis();
    bevy::log::debug!("Computed dijkstra map in {} ms", elapsed);

    distances
}
