use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::components::WorldPos;

pub const TILE_SIZE: f32 = 32.0;

pub const MAP_WIDTH_TILES: i32 = 41;
pub const MAP_HEIGHT_TILES: i32 = 41;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TileType {
    Wall,
    Floor,
}

impl TileType {
    pub fn blocks_visibility(&self) -> bool {
        match *self {
            TileType::Wall => true,
            TileType::Floor => false,
        }
    }

    pub fn blocks_movement(&self) -> bool {
        match *self {
            TileType::Wall => true,
            TileType::Floor => false,
        }
    }
}

/// Bounding box (also used as a rectangle). All coordinates are in world (tile) coordinates,
/// and all coordinates are inclusive.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BoundingBox {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

impl BoundingBox {
    pub fn make_empty() -> Self {
        BoundingBox {
            x_min: i32::MAX,
            y_min: i32::MAX,
            x_max: i32::MIN,
            y_max: i32::MIN,
        }
    }

    pub fn include_pt(&mut self, wp: WorldPos) {
        self.x_min = self.x_min.min(wp.x);
        self.x_max = self.x_max.max(wp.x);
        self.y_min = self.y_min.min(wp.y);
        self.y_max = self.y_max.max(wp.y);
    }

    /// Checks whether two boxes intersect each other (have any tiles in common).
    ///
    /// The required_buffer parameter requires that many squares of "unrelated" / neutral space
    /// between them. So zero just means the boxes can't overlap; one means there must be a
    /// square between them, and so on.
    pub fn intersects(&self, other: &BoundingBox, required_buffer: i32) -> bool {
        if self.empty() || other.empty() {
            return false;
        }

        let one_dim_intersection = |a_min: i32, a_max: i32, b_min: i32, b_max: i32| {
            let left = a_min.max(b_min);
            let right = a_max.min(b_max);
            left - required_buffer <= right
        };

        let out = one_dim_intersection(self.x_min, self.x_max, other.x_min, other.x_max)
            && one_dim_intersection(self.y_min, self.y_max, other.y_min, other.y_max);

        out
    }

    pub fn width(&self) -> i32 {
        if self.empty() {
            0
        } else {
            self.x_max - self.x_min + 1
        }
    }

    pub fn height(&self) -> i32 {
        if self.empty() {
            0
        } else {
            self.y_max - self.y_min + 1
        }
    }

    pub fn empty(&self) -> bool {
        self.x_min > self.x_max || self.y_min > self.y_max
    }

    pub fn center(&self) -> (i32, i32) {
        if self.empty() {
            panic!("Cannot get center of empty room");
        }

        return (
            self.x_min + self.width() / 2,
            self.y_min + self.height() / 2,
        );
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox::make_empty()
    }
}

pub struct Map {
    // TODO perf: HashMap is probably not sustainable but it solves a lot of indexing problems
    default_tile: TileType,
    tiles: HashMap<WorldPos, TileType>,
    rooms: Vec<BoundingBox>,
    bounds: BoundingBox,
    visible: HashSet<WorldPos>,
    seen: HashSet<WorldPos>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            default_tile: TileType::Wall,
            tiles: HashMap::new(),
            rooms: Vec::new(),
            bounds: BoundingBox::default(),
            seen: HashSet::new(),
            visible: HashSet::new(),
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    pub fn get_tile(&self, wp: WorldPos) -> TileType {
        self.tiles.get(&wp).copied().unwrap_or(self.default_tile)
    }

    pub fn passable(&self, wp: WorldPos) -> bool {
        !self.get_tile(wp).blocks_movement()
    }

    pub fn set_if_empty(&mut self, wp: WorldPos, tile: TileType) {
        // This feels like it should be a normal hashmap thing but who knows
        self.tiles.entry(wp).or_insert(tile);
        self.bounds.include_pt(wp);
    }

    pub fn set_tile(&mut self, wp: WorldPos, tile: TileType) {
        for x in wp.x - 1..wp.x + 2 {
            for y in wp.y - 1..wp.y + 2 {
                self.set_if_empty(WorldPos { x, y }, self.default_tile);
            }
        }
        self.tiles.insert(wp, tile);
        self.bounds.include_pt(wp);
    }

    pub fn tiles(&self) -> Box<dyn Iterator<Item = TileData> + '_> {
        let out = self.tiles.iter().map(|(wp, tt)| TileData {
            world_pos: *wp,
            tile_type: *tt,
            seen: self.seen.contains(wp),
            visible: self.visible.contains(wp),
        });
        Box::new(out)
    }

    /// Returns adjacent tiles which are passable
    pub fn adjacent(&self, wp: WorldPos) -> Box<dyn Iterator<Item = WorldPos> + '_> {
        let WorldPos { x, y } = wp;
        let out = [(x, y - 1), (x - 1, y), (x, y + 1), (x + 1, y)]
            .into_iter()
            .map(|(x, y)| WorldPos { x, y })
            .filter(|wp| {
                let tt = self.tiles.get(wp).copied().unwrap_or(self.default_tile);
                !tt.blocks_movement()
            });
        Box::new(out)
    }

    pub fn mark_visible(&mut self, wp: WorldPos) {
        self.visible.insert(wp);
        self.seen.insert(wp);
    }

    pub fn set_visible_exact(&mut self, visible: &HashSet<WorldPos>) {
        self.visible.clear();

        for wp in visible {
            self.mark_visible(*wp);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TileData {
    pub world_pos: WorldPos,
    pub tile_type: TileType,
    pub seen: bool,
    pub visible: bool,
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

pub fn make_new_map() -> (Map, Vec<BoundingBox>) {
    let mut map = Map::new();

    const MIN_SIZE: i32 = 3;
    const MAX_SIZE: i32 = 5;
    const MAX_ROOMS: i32 = 30;

    let mut rng = rand::thread_rng();
    let mut rooms: Vec<BoundingBox> = Vec::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let h = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let x = rng.gen_range(1..MAP_WIDTH_TILES - w - 1);
        let y = rng.gen_range(1..MAP_HEIGHT_TILES - h - 1);

        let room_box = BoundingBox {
            x_min: x,
            x_max: x + w - 1,
            y_min: y,
            y_max: y + h - 1,
        };

        // TODO: needs a border; this allows two rooms' walls to be inside the other one's room
        let mut ok = true;
        for other in &rooms {
            if other.intersects(&room_box, 1) {
                ok = false;
                break;
            }
        }

        if ok {
            rooms.push(room_box);
            apply_room_to_map(&mut map, room_box);
        }
    }

    for i in 0..rooms.len() - 1 {
        let (old_x, old_y) = rooms[i].center();
        let (new_x, new_y) = rooms[i + 1].center();
        if rng.gen_range(0..2) == 0 {
            apply_horizontal_tunnel(&mut map, old_x, new_x, old_y);
            apply_vertical_tunnel(&mut map, old_y, new_y, new_x);
        } else {
            apply_vertical_tunnel(&mut map, old_y, new_y, old_x);
            apply_horizontal_tunnel(&mut map, old_x, new_x, new_y);
        }
    }

    (map, rooms)
}

fn apply_room_to_map(map: &mut Map, room: BoundingBox) {
    for x in room.x_min..room.x_max + 1 {
        for y in room.y_min..room.y_max + 1 {
            map.set_tile(WorldPos { x, y }, TileType::Floor);
        }
    }

    map.rooms.push(room);
}

fn apply_horizontal_tunnel(map: &mut Map, mut old_x: i32, mut new_x: i32, y: i32) {
    if old_x > new_x {
        std::mem::swap(&mut old_x, &mut new_x);
    }

    for x in old_x..new_x + 1 {
        map.set_tile(WorldPos { x, y }, TileType::Floor);
    }
}

fn apply_vertical_tunnel(map: &mut Map, mut old_y: i32, mut new_y: i32, x: i32) {
    if old_y > new_y {
        std::mem::swap(&mut old_y, &mut new_y);
    }

    for y in old_y..new_y + 1 {
        map.set_tile(WorldPos { x, y }, TileType::Floor);
    }
}
