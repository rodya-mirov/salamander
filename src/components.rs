use std::collections::HashSet;
use bevy::prelude::Entity;

use crate::map::TileType;

/// Marker struct indicating this entity is the player camera (so the camera should center on it)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerCamera;

/// Marker struct that this entity is the player
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Player;

/// Position in the world (as opposed to a Bevy graphical transform)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
}

impl std::fmt::Display for WorldPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Anything that needs a name, I guess
pub struct EntityName(pub String);

/// Marker struct that an entity should be managed by a Monster AI
pub struct MonsterAI;

/// Indicator that an entity prevents movement. Affects pathing.
pub struct BlocksMovement;

/// Marker struct that an entity is a visual representation of a tile
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisualTile(pub TileType);

/// Marker struct indicating that an entity should not be displayed if it is not currently being
/// looked at.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RequiresSeen;

/// Component describing a Viewshed, literally the set of tiles that are visible
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Viewshed {
    pub visible_tiles: HashSet<WorldPos>,
    // TODO perf: revive this if FOV computations become a problem
    // pub dirty: bool,
    pub range: i32,
}

impl Viewshed {
    pub fn new() -> Self {
        Viewshed {
            visible_tiles: HashSet::new(),
            range: 7,
        }
    }
}

/// Event indicating the map has changed, to indicate that stuff needs to be rebuilt
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MapChangedEvent;

/// Event indicating something about visibility has changed, to indicate that visual stuff needs to be rebuilt
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisibilityChangedEvent;

/// Event indicating a player has finished their turn; used for flow control (indicating the player has finished their turn)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerTookTurnEvent;

/// Event indicating an entity moved
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntityMovedEvent { pub old_pos: WorldPos, pub new_pos: WorldPos, pub entity: Entity}

