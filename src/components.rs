use crate::resources::TileType;

pub const TILE_SIZE: f32 = 32.0;

pub const MAP_WIDTH_TILES: i32 = 31;
pub const MAP_HEIGHT_TILES: i32 = 31;

/// Marker struct indicating this entity is the player camera (so the camera should center on it)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PlayerCamera;

/// Marker struct that this entity is the player
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Player;

/// Position in the world (as opposed to a Bevy graphical transform)
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
}

/// Marker struct that an entity is a visual representation of a tile
pub struct VisualTile(pub TileType);

/// Event indicating the map has changed, to indicate that stuff needs to be rebuilt
pub struct MapChangedEvent;
