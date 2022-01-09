use std::collections::HashSet;

use bevy::prelude::*;

use crate::map::TileType;

/// Marker struct indicating this entity is the player camera (so the camera should center on it)
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerCamera;

/// Marker struct that this entity is the player
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Player;

/// Position in the world (as opposed to a Bevy graphical transform)
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
#[derive(Component)]
pub struct EntityName(pub String);

/// Marker struct that an entity should be managed by a Monster AI
#[derive(Component)]
pub struct MonsterAI;

/// Indicator that an entity prevents movement. Affects pathing.
#[derive(Component)]
pub struct BlocksMovement;

/// Marker struct that an entity is a visual representation of a tile
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisualTile(pub TileType);

/// Marker struct indicating that an entity should not be displayed if it is not currently being
/// looked at.
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct RequiresSeen;

/// Component describing a Viewshed, literally the set of tiles that are visible
#[derive(Component, Clone, Eq, PartialEq, Debug)]
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
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MapChangedEvent;

/// Event indicating something about visibility has changed, to indicate that visual stuff needs to be rebuilt
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisibilityChangedEvent;

/// Event indicating a player has finished their turn; used for flow control (indicating the player has finished their turn)
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayerTookTurnEvent;

/// Event indicating an entity moved
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntityMovedEvent {
    pub old_pos: WorldPos,
    pub new_pos: WorldPos,
    pub entity: Entity,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
pub struct WantsToMelee {
    pub attacker: Entity,
    pub defender: Entity,
}
