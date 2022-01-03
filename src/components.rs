use std::collections::HashSet;

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

/// Marker struct that an entity is a visual representation of a tile
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisualTile(pub TileType);

/// Component describing a Viewshed, literally the set of tiles that are visible
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Viewshed {
    pub visible_tiles: HashSet<WorldPos>,
    pub dirty: bool,
    pub range: i32,
}

impl Viewshed {
    pub fn new(range: i32) -> Self {
        Viewshed {
            visible_tiles: HashSet::new(),
            dirty: true,
            range,
        }
    }
}

/// Event indicating the map has changed, to indicate that stuff needs to be rebuilt
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MapChangedEvent;

/// Event indicating something about visibility has changed, to indicate that stuff needs to be rebuilt
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VisibilityChangedEvent;
