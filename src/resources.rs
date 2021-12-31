use std::collections::HashMap;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerInputState {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub right_pressed: bool,
    pub left_pressed: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    // TODO perf: HashMap is probably not sustainable but it solves a lot of indexing problems
    tiles: HashMap<(i32, i32), TileType>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            tiles: HashMap::new(),
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        self.tiles.get(&(x, y)).copied()
    }

    pub fn passable(&self, x: i32, y: i32) -> bool {
        let tile = self.get_tile(x, y);
        if tile.is_none() {
            false
        } else {
            match tile.unwrap() {
                TileType::Wall => false,
                TileType::Floor => true,
            }
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        self.tiles.insert((x, y), tile);
    }

    pub fn tiles(&self) -> Box<dyn Iterator<Item = ((i32, i32), TileType)> + '_> {
        let out = self.tiles.iter().map(|(k, v)| (*k, *v));
        Box::new(out)
    }
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}
