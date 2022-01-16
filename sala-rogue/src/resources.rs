use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use bevy::prelude::*;

use crate::components::*;

pub mod events;

/// Just tracks the current turn number of the game. Used to age logs and stuff.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CurrentTurnNumber(pub usize);

/// Indicates the player system has already run once this frame, which is used for various things
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerMovedInFrame(pub bool);

/// Indicates the player system has generated a "no action" choice, or has been early stopped,
/// which is used for various things
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerNoAction(pub bool);

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerInputState {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub right_pressed: bool,
    pub left_pressed: bool,
    pub pass_pressed: bool,
}

#[derive(Default, Debug)]
pub struct Logs {
    /// logs[0] is the newest
    logs: VecDeque<LogInfo>,
}

impl Logs {
    pub fn push(&mut self, log: LogInfo) {
        self.logs.push_front(log);
    }

    pub fn iter(&self, range: impl Iterator<Item = usize>) -> impl Iterator<Item = &LogInfo> {
        range.map(|i| self.logs.get(i)).flatten()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogInfo {
    pub log: Log,
    pub issue_round: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Log {
    pub message: String,
}

#[derive(Default, Debug)]
pub struct TurnOrder {
    /// Everything gets a turn -- players, mobs, environmental effects, whatever
    /// And everything goes in order
    turn_order: VecDeque<Entity>,
}

impl TurnOrder {
    pub fn current_holder(&self) -> Option<Entity> {
        self.turn_order.get(0).copied()
    }

    pub fn end_turn(&mut self) {
        if self.turn_order.len() <= 1 {
            return;
        }
        self.turn_order.rotate_left(1);
    }

    pub fn add_if_not_present(&mut self, entity: Entity) {
        let exists = self.turn_order.iter().copied().any(|e| e == entity);
        if !exists {
            self.turn_order.push_back(entity);
        }
    }

    pub fn remove_from_turn_order(&mut self, entity: Entity) {
        let mut i = 0;
        while i < self.turn_order.len() {
            if Some(entity) == self.turn_order.get(i).copied() {
                break;
            }
            i += 1;
        }

        if i < self.turn_order.len() {
            self.turn_order.remove(i);
        }
    }

    #[allow(dead_code)] // used for debug stuff
    pub fn len(&self) -> usize {
        self.turn_order.len()
    }
}

pub struct BasicTilesAtlas(pub Handle<TextureAtlas>);

#[derive(Default, Clone, Debug)]
pub struct CacheMap(HashMap<WorldPos, HashSet<Entity>>);

impl CacheMap {
    pub fn remove_entity(&mut self, wp: WorldPos, entity: Entity) {
        self.0.entry(wp).or_default().remove(&entity);
    }

    pub fn remove_entity_anywhere(&mut self, entity: Entity) {
        self.0.values_mut().for_each(|s| {
            s.remove(&entity);
        });
    }

    pub fn update_entity(&mut self, old_wp: WorldPos, new_wp: WorldPos, entity: Entity) {
        self.remove_entity(old_wp, entity);
        self.add_entity(new_wp, entity);
    }

    pub fn add_entity(&mut self, wp: WorldPos, entity: Entity) {
        self.0.entry(wp).or_default().insert(entity);
    }

    pub fn has_any(&self, wp: WorldPos) -> bool {
        self.0.get(&wp).map(|s| !s.is_empty()).unwrap_or(false)
    }

    pub fn get_any(&self, wp: WorldPos) -> Option<Entity> {
        match self.0.get(&wp) {
            Some(set) => set.iter().copied().next(),
            None => None,
        }
    }
}

pub use events::{CallbackEvent, CallbackEvents};

#[derive(Default, Clone, Debug)]
pub struct BlockedTiles(pub CacheMap);

#[derive(Default, Clone, Debug)]
pub struct CombatStatsTiles(pub CacheMap);

// TODO: make this a macro, I think we're gonna do it a lot
impl std::ops::Deref for BlockedTiles {
    type Target = CacheMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BlockedTiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for CombatStatsTiles {
    type Target = CacheMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CombatStatsTiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type DijkstraMap = HashMap<WorldPos, i32>;

#[derive(Default, Clone, Debug)]
pub struct PlayerDistanceMap(pub DijkstraMap);
