use bevy::prelude::*;

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

pub fn handle_input(input: Res<PlayerInputState>, mut q: Query<(&Player, &mut WorldPos)>) {
    for (_, mut wp) in q.iter_mut() {
        if input.left_pressed {
            wp.x -= 1;
        }
        if input.right_pressed {
            wp.x += 1;
        }
        if input.up_pressed {
            wp.y += 1;
        }
        if input.down_pressed {
            wp.y -= 1;
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
