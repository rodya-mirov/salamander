use bevy::prelude::*;

use crate::bevy_util::make_text_bundle;
use crate::components::*;
use crate::resources::*;

pub fn get_player_input(kb_input: Res<Input<KeyCode>>, mut input_state: ResMut<PlayerInputState>) {
    *input_state = PlayerInputState::default();

    if kb_input.just_pressed(KeyCode::A) || kb_input.just_pressed(KeyCode::Left) {
        input_state.left_pressed = true;
    }

    if kb_input.just_pressed(KeyCode::D) || kb_input.just_pressed(KeyCode::Right) {
        input_state.right_pressed = true;
    }

    if kb_input.just_pressed(KeyCode::W) || kb_input.just_pressed(KeyCode::Up) {
        input_state.up_pressed = true;
    }

    if kb_input.just_pressed(KeyCode::S) || kb_input.just_pressed(KeyCode::Down) {
        input_state.down_pressed = true;
    }

    if input_state.up_pressed && input_state.down_pressed {
        input_state.up_pressed = false;
        input_state.down_pressed = false;
    }

    if input_state.left_pressed && input_state.right_pressed {
        input_state.left_pressed = false;
        input_state.right_pressed = false;
    }
}

pub fn handle_input(
    input: Res<PlayerInputState>,
    mut q: Query<(&Player, &mut WorldPos)>,
    map: Res<Map>,
) {
    for (_, mut wp) in q.iter_mut() {
        let mut new_wp = *wp;
        if input.left_pressed {
            new_wp.x -= 1;
        }
        if input.right_pressed {
            new_wp.x += 1;
        }
        if input.up_pressed {
            new_wp.y += 1;
        }
        if input.down_pressed {
            new_wp.y -= 1;
        }
        if map.passable(new_wp.x, new_wp.y) {
            *wp = new_wp;
        }
    }
}

pub fn aim_camera(
    player_query: Query<(&Player, &WorldPos)>,
    mut camera_query: Query<(&PlayerCamera, &mut Transform)>,
) {
    // we need the player's position to center the camera
    let player_wp: WorldPos = *player_query.single().expect("Player must exist").1;

    // aim the camera at the player
    for (_, mut transform) in camera_query.iter_mut() {
        transform.translation.x = player_wp.x as f32 * TILE_SIZE;
        transform.translation.y = player_wp.y as f32 * TILE_SIZE;
    }
}

pub fn world_pos_to_visual_system(mut wp_query: Query<(&WorldPos, &mut Transform)>) {
    // lock everything to their world position (that is, graphical transform is derived from WP)
    for (wp, mut transform) in wp_query.iter_mut() {
        let wp: WorldPos = *wp;
        transform.translation.x = wp.x as f32 * TILE_SIZE;
        transform.translation.y = wp.y as f32 * TILE_SIZE;
    }
}

pub fn rebuild_visual_tiles(
    mut commands: Commands,
    mut rebuild_events: EventReader<MapChangedEvent>,
    q: Query<(Entity, &VisualTile)>,
    map: Res<Map>,
    asset_server: Res<AssetServer>,
) {
    // even if we send a zillion, we only rebuild once
    if rebuild_events.iter().next().is_none() {
        return;
    }

    // wipe out anything previous existing, if any
    for (e, _) in q.iter() {
        commands.entity(e).despawn();
    }

    // then build new tiles
    for ((x, y), tile_type) in map.tiles() {
        // TODO: use the tileset because it's awesome
        let sigil = match tile_type {
            TileType::Wall => 'X',
            TileType::Floor => '.',
        };

        commands
            .spawn()
            .insert(VisualTile(tile_type))
            .insert_bundle(make_text_bundle(sigil, &asset_server))
            .insert(WorldPos { x, y })
            .insert(Transform::default());
    }
}

pub fn noop_system() {
    // so the stage is nonempty
}
