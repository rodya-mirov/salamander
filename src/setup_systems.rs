use bevy::prelude::*;

use crate::bevy_util::make_basic_sprite_bundle;
use crate::components::*;
use crate::map::*;
use crate::resources::*;

pub fn make_map(
    mut map_res: ResMut<Map>,
    mut map_events: EventWriter<MapChangedEvent>,
    mut commands: Commands,
    sheet: Res<BasicTilesAtlas>,
) {
    let (map, rooms) = make_new_map();

    let room = rooms[0];
    let (x, y) = room.center();

    commands
        .spawn()
        .insert(Player)
        .insert_bundle(make_basic_sprite_bundle(2, &sheet.0, Color::ALICE_BLUE))
        .insert(Viewshed::new(7))
        .insert(WorldPos { x, y })
        // TODO: coherent layering management system, not like this
        .insert(Transform::from_xyz(0.0, 0.0, 100.0));

    *map_res = map;

    map_events.send(MapChangedEvent);
}

pub fn load_tileset(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let sheet_handle: Handle<Texture> = asset_server.load("tiles/basic_tiles.png");
    commands.insert_resource(BasicTilesSheet(sheet_handle.clone()));
    let texture_atlas = TextureAtlas::from_grid(sheet_handle, Vec2::new(32.0, 32.0), 16, 20);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(BasicTilesAtlas(texture_atlas_handle));
}
