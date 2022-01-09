use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::bevy_util::make_basic_sprite_bundle;
use crate::components::*;
use crate::map::{Map, TileType, TILE_SIZE};
use crate::resources::*;

mod dijkstra;
mod fov;

pub use fov::{compute_viewsheds, update_map_visibility};

pub fn get_player_input(kb_input: Res<Input<KeyCode>>, mut input_state: ResMut<PlayerInputState>) {
    *input_state = PlayerInputState::default();

    if kb_input.any_just_pressed([KeyCode::A, KeyCode::Left, KeyCode::Numpad4]) {
        input_state.left_pressed = true;
    }

    if kb_input.any_just_pressed([KeyCode::D, KeyCode::Right, KeyCode::Numpad6]) {
        input_state.right_pressed = true;
    }

    if kb_input.any_just_pressed([KeyCode::W, KeyCode::Up, KeyCode::Numpad8]) {
        input_state.up_pressed = true;
    }

    if kb_input.any_just_pressed([KeyCode::S, KeyCode::Down, KeyCode::Numpad2]) {
        input_state.down_pressed = true;
    }

    if kb_input.just_pressed(KeyCode::Space) {
        input_state.pass_pressed = true;
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
    mut qs: QuerySet<(
        QueryState<(&mut WorldPos, Entity), With<Player>>,
        // we can keep this in another resource but it's becoming really tedious to maintain
        // so many things in one place (again just imagining the monster AI loop)
        // we could simplify things by making the game turn-based and giving EACH enemy a turn
        // and having each tick be one turn (unless we're waiting for the player)
        // but this might make the game feel really sluggish if there are 60 NPCs on the map
        // realistically we can probably get >1 turn in a single timestep but programmatically
        // I don't know how to achieve that
        QueryState<(&WorldPos, Entity), With<CombatStats>>,
    )>,
    map: Res<Map>,
    mut blocked: ResMut<BlockedTiles>,
    mut player_map: ResMut<PlayerDistanceMap>,
    mut turn_events: EventWriter<PlayerTookTurnEvent>,
    mut moved_events: EventWriter<EntityMovedEvent>,
    mut attack_events: EventWriter<WantsToMelee>,
) {
    let mut cs_positions: HashMap<WorldPos, Entity> = HashMap::new();
    for (wp, entity) in qs.q1().iter() {
        cs_positions.insert(*wp, entity);
    }
    for (mut wp, entity) in qs.q0().iter_mut() {
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

        if input.pass_pressed {
            turn_events.send(PlayerTookTurnEvent);
        } else if new_wp != *wp {
            if let Some(&defender) = cs_positions.get(&new_wp) {
                turn_events.send(PlayerTookTurnEvent);
                attack_events.send(WantsToMelee {
                    attacker: entity,
                    defender,
                });
            } else if can_pass(new_wp, &*map, &*blocked) {
                turn_events.send(PlayerTookTurnEvent);
                blocked.update_block(*wp, new_wp);
                moved_events.send(EntityMovedEvent {
                    entity,
                    old_pos: *wp,
                    new_pos: new_wp,
                });
                *wp = new_wp;

                // We don't need to access the Blocked resource here, better to let the monsters pile
                // up and get stuck if necessary
                let new_player_map =
                    dijkstra::distance_dijkstra_map(&*map, [new_wp].iter(), |_| false);
                player_map.0 = new_player_map;
            }
        }
    }
}

fn can_pass(new_wp: WorldPos, map: &Map, blocked: &BlockedTiles) -> bool {
    map.passable(new_wp) && !blocked.is_blocked(new_wp)
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
    let player_wp: WorldPos = match player_query.get_single() {
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

/// Make the visible (bevy rendering) component reflect the actual viewing state, if relevant
pub fn hide_unseen_things(
    player_query: Query<&Viewshed, (With<Player>,)>,
    mut to_hide_query: Query<(&WorldPos, &mut Visibility), (With<RequiresSeen>,)>,
) {
    let player_vs = match player_query.iter().next() {
        Some(vs) => vs.visible_tiles.clone(),
        None => HashSet::new(),
    };

    for (wp, mut visible) in to_hide_query.iter_mut() {
        if player_vs.contains(&*wp) {
            visible.is_visible = true;
        } else {
            visible.is_visible = false;
        }
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

pub fn monster_ai(
    mut player_moved: EventReader<PlayerTookTurnEvent>,
    mut entity_moves: EventWriter<EntityMovedEvent>,
    mut query_set: QuerySet<(
        QueryState<&WorldPos, With<Player>>,
        QueryState<(Option<&EntityName>, &mut Viewshed, &mut WorldPos, Entity), With<MonsterAI>>,
    )>,
    map: Res<Map>,
    mut blocked: ResMut<BlockedTiles>,
    player_map: Res<PlayerDistanceMap>,
) {
    if player_moved.iter().next().is_none() {
        // don't bother
        return;
    }

    let player_pos: WorldPos = match query_set.q0().iter().next() {
        Some(wp) => *wp,
        // no player no action
        None => return,
    };

    for (maybe_name, vs, mut wp, entity) in query_set.q1().iter_mut() {
        let name = maybe_name
            .as_ref()
            .map(|n| n.0.as_str())
            .unwrap_or("Monster");

        if vs.visible_tiles.contains(&player_pos) {
            println!(
                "{} at {} can see the player at {}! zomg",
                name, *wp, player_pos
            );
        } else {
            // player is not visible; do nothing
            // don't chase them all over the map, only if they can be seen
            continue;
        }

        let curr_dist = player_map.0.get(&*wp).copied().unwrap_or(i32::MAX);

        let mut new_wp = *wp;
        let mut best_dest = curr_dist;

        for adj_tile in map.adjacent(*wp).filter(|w| !blocked.is_blocked(*w)) {
            let adj_dist = player_map.0.get(&adj_tile).copied().unwrap_or(i32::MAX);
            if adj_dist < best_dest {
                best_dest = adj_dist;
                new_wp = adj_tile;
            }
        }

        if new_wp != *wp {
            blocked.update_block(*wp, new_wp);
            entity_moves.send(EntityMovedEvent {
                entity,
                old_pos: *wp,
                new_pos: new_wp,
            });
            *wp = new_wp;
        }
    }
}
