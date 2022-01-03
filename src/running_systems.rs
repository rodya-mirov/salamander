use bevy::prelude::*;

use crate::bevy_util::make_basic_sprite_bundle;
use crate::components::*;
use crate::map::{Map, TileType, TILE_SIZE};
use crate::resources::*;

mod fov;

pub use fov::{compute_viewsheds, update_map_visibility};

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
    mut q: Query<(&Player, &mut WorldPos, Option<&mut Viewshed>)>,
    map: Res<Map>,
) {
    for (_, mut wp, vs) in q.iter_mut() {
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
        if *wp != new_wp && map.passable(new_wp) {
            *wp = new_wp;

            if let Some(mut vs) = vs {
                vs.dirty = true;
            }
        }
    }
}

pub fn aim_camera(
    window: Res<WindowDescriptor>,
    map: Res<Map>,
    player_query: Query<(&Player, &WorldPos)>,
    mut camera_query: Query<(&PlayerCamera, &mut Transform)>,
) {
    fn get_desired_wp_pt(
        player_wp_pt: i32,
        map_min_wp: i32,
        map_max_wp: i32,
        window_size_px: f32,
    ) -> f32 {
        let window_size_wp = window_size_px / TILE_SIZE;

        // less than this and we have empty space on the left
        let camera_min_wp = map_min_wp as f32 + (window_size_wp / 2.0) - 0.5;

        // more than this and we have empty space on the right
        let camera_max_wp = map_max_wp as f32 - (window_size_wp / 2.0) + 0.5;

        // quick check: min_wp >= max_wp iff window_size >= map_width
        // in which case just center the map
        let desired_wp = player_wp_pt as f32;
        if camera_min_wp >= camera_max_wp {
            (map_min_wp + map_max_wp) as f32 / 2.0
        } else if desired_wp < camera_min_wp {
            camera_min_wp
        } else if desired_wp > camera_max_wp {
            camera_max_wp
        } else {
            desired_wp
        }
    }

    // we need the player's position to center the camera
    let player_wp: WorldPos = match player_query.single() {
        Ok(player) => *player.1,
        // if no player, just end the system
        Err(_) => {
            return;
        }
    };

    let bounds = map.bounding_box();

    let desired_x_wp = get_desired_wp_pt(player_wp.x, bounds.x_min, bounds.x_max, window.width);
    let desired_y_wp = get_desired_wp_pt(player_wp.y, bounds.y_min, bounds.y_max, window.height);

    // aim the camera at the player
    for (_, mut transform) in camera_query.iter_mut() {
        let x_dist = desired_x_wp * TILE_SIZE - transform.translation.x;
        let y_dist = desired_y_wp * TILE_SIZE - transform.translation.y;

        transform.translation.x += x_dist;
        transform.translation.y += y_dist;
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
    sheet: Res<BasicTilesAtlas>,
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
    for tile_data in map.tiles() {
        let tile_idx = match tile_data.tile_type {
            TileType::Wall => 8 * 16 + 3,
            TileType::Floor => 7 * 16 + 8,
        };

        if tile_data.seen {
            let color = if tile_data.visible {
                match tile_data.tile_type {
                    TileType::Floor => Color::rgb(0.4, 0.75, 0.4),
                    TileType::Wall => Color::rgb(0.8, 0.79, 0.57),
                }
            } else {
                Color::GRAY
            };
            commands
                .spawn()
                .insert(VisualTile(tile_data.tile_type))
                .insert_bundle(make_basic_sprite_bundle(tile_idx, &sheet.0, color))
                .insert(tile_data.world_pos)
                // TODO: tile layers?
                .insert(Transform::default());
        }
    }
}

pub fn noop_system() {
    // so the stage is nonempty
}
