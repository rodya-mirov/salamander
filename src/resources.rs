use std::collections::HashMap;

use bevy::prelude::*;

use crate::components::*;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerInputState {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub pass_pressed: bool,
}

pub struct BasicTilesSheet(pub Handle<Texture>);

pub struct BasicTilesAtlas(pub Handle<TextureAtlas>);

#[derive(Default, Clone, Debug)]
pub struct BlockedTiles(HashMap<WorldPos, i32>);

impl BlockedTiles {
    pub fn remove_block(&mut self, wp: WorldPos) {
        *self.0.entry(wp).or_default() -= 1;
    }

    pub fn update_block(&mut self, old_wp: WorldPos, new_wp: WorldPos) {
        self.remove_block(old_wp);
        self.add_block(new_wp);
    }

    pub fn add_block(&mut self, wp: WorldPos) {
        *self.0.entry(wp).or_default() += 1;
    }

    pub fn is_blocked(&self, wp: WorldPos) -> bool {
        self.0.get(&wp).map(|s| *s > 0).unwrap_or(false)
    }
}

pub type DijkstraMap = HashMap<WorldPos, i32>;

#[derive(Default, Clone, Debug)]
pub struct PlayerDistanceMap(pub DijkstraMap);
