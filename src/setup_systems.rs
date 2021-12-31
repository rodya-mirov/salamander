use bevy::prelude::*;
use rand::Rng;

use crate::bevy_util::make_text_bundle;
use crate::components::{Player, WorldPos, MAP_HEIGHT_TILES, MAP_WIDTH_TILES};
use crate::resources::{Map, TileType};
use crate::MapChangedEvent;

pub fn make_map(mut map_res: ResMut<Map>, mut map_events: EventWriter<MapChangedEvent>) {
    let mut map = Map::new();
    for x in 0..MAP_WIDTH_TILES {
        map.set_tile(x, 0, TileType::Wall);
        map.set_tile(x, MAP_HEIGHT_TILES - 1, TileType::Wall);
    }
    for y in 0..MAP_HEIGHT_TILES {
        map.set_tile(0, y, TileType::Wall);
        map.set_tile(MAP_WIDTH_TILES - 1, y, TileType::Wall);
    }

    let mut rng = rand::thread_rng();

    for x in 1..(MAP_WIDTH_TILES - 1) {
        for y in 1..(MAP_HEIGHT_TILES - 1) {
            let tt = if rng.gen_bool(0.15) {
                TileType::Wall
            } else {
                TileType::Floor
            };
            map.set_tile(x, y, tt);
        }
    }

    *map_res = map;

    map_events.send(MapChangedEvent);
}

pub fn world_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(make_text_bundle('@', &asset_server))
        .insert(WorldPos {
            x: (MAP_WIDTH_TILES - 1) / 2,
            y: (MAP_HEIGHT_TILES - 1) / 2,
        })
        // TODO: coherent layering management system, not like this
        .insert(Transform::from_xyz(0.0, 0.0, 100.0));
}
