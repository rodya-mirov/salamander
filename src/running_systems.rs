use std::collections::HashSet;

use bevy::app::Events;
use bevy::diagnostic::Diagnostics;
use bevy::prelude::*;

use crate::bevy_util::make_basic_sprite_bundle;
use crate::components::*;
use crate::map::{Map, TileType, TILE_SIZE};
use crate::resources::*;
use crate::{AppExtension, FrameTimeDiagnosticsPlugin};

mod dijkstra;
mod fov;

pub use fov::{compute_viewsheds, update_map_visibility};

pub fn world_tick(world: &mut World) {
    // This is done once at the top of the tick, not inside the loop
    let mut input_state = SystemStage::single_threaded();
    input_state.add_system(get_player_input);
    input_state.add_system(clear_player_moved_in_frame);
    input_state.run(world);

    // Single threaded isn't enough to guarantee execution order, so it's still super janky
    let mut system_idx = 0;
    let mut full_stage = SystemStage::single_threaded();

    // TODO: is there a reason we might want to keep this SystemStage in some kind of cached / stored place in the app? i guess profile?
    full_stage
        // first, make sure turn order and map registration are set up correctly
        .add_sequential_system(&mut system_idx, assign_turn_order)
        .add_sequential_system(&mut system_idx, clear_turn_order_requests)
        .add_sequential_system(&mut system_idx, assign_block_map_indexing)
        .add_sequential_system(&mut system_idx, assign_combat_map_indexing)
        .add_sequential_system(&mut system_idx, clear_indexing_requests)
        // then let thinking agents take their turns
        .add_sequential_system(&mut system_idx, handle_input)
        .add_sequential_system(&mut system_idx, monster_ai)
        // then, cleanup systems
        .add_sequential_system(&mut system_idx, process_combat_event)
        .add_sequential_system(&mut system_idx, process_suffers_damage_event)
        .add_sequential_system(&mut system_idx, update_blocked_map)
        .add_sequential_system(&mut system_idx, update_combat_stats_map)
        .add_sequential_system(&mut system_idx, compute_viewsheds)
        .add_sequential_system(&mut system_idx, update_map_visibility)
        .add_sequential_system(&mut system_idx, rebuild_visual_tiles)
        .add_sequential_system(&mut system_idx, death_system)
        .add_sequential_system(&mut system_idx, remove_dead_from_maps)
        // finally, let the next entity take their turn
        .add_sequential_system(&mut system_idx, next_turn)
        .add_sequential_system(&mut system_idx, drain_turn_events);

    let start = std::time::Instant::now();
    let budget_ms = 6000; // 12 ms for this system keeps us at a healthy 60 fps with 4ms left for rendering :grimace:

    while start.elapsed().as_millis() < budget_ms {
        full_stage.run(world);

        // In the extremely common case where the player comes up twice, or they took no action,
        // we know nothing else is going to happen and we can stop immediately
        if world
            .get_resource::<PlayerNoAction>()
            .map(|p| p.0)
            .unwrap_or(false)
        {
            break;
        }
    }
}

pub fn clear_player_moved_in_frame(
    mut moved_in_frame: ResMut<PlayerMovedInFrame>,
    mut no_action: ResMut<PlayerNoAction>,
) {
    moved_in_frame.0 = false;
    no_action.0 = false;
}

pub fn assign_turn_order(
    q: Query<Entity, With<WantsTurnOrderAssignment>>,
    mut turns: ResMut<TurnOrder>,
) {
    for entity in q.iter() {
        turns.add_if_not_present(entity);
    }
}

pub fn clear_turn_order_requests(
    mut commands: Commands,
    q: Query<Entity, With<WantsTurnOrderAssignment>>,
) {
    for entity in q.iter() {
        commands.entity(entity).remove::<WantsTurnOrderAssignment>();
    }
}

// TODO: make a macro for this
pub fn assign_block_map_indexing(
    q: Query<(Entity, &WorldPos), (With<WantsMapIndexing>, With<BlocksMovement>)>,
    mut blocks: ResMut<BlockedTiles>,
) {
    for (entity, wp) in q.iter() {
        let wp = *wp;
        blocks.add_entity(wp, entity);
    }
}

pub fn assign_combat_map_indexing(
    q: Query<(Entity, &WorldPos), (With<WantsMapIndexing>, With<CombatStats>)>,
    mut blocks: ResMut<CombatStatsTiles>,
) {
    for (entity, wp) in q.iter() {
        let wp = *wp;
        blocks.add_entity(wp, entity);
    }
}

pub fn clear_indexing_requests(mut commands: Commands, q: Query<Entity, With<WantsMapIndexing>>) {
    for entity in q.iter() {
        commands.entity(entity).remove::<WantsMapIndexing>();
    }
}

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
    mut player_lock: ResMut<PlayerMovedInFrame>,
    mut player_no_action: ResMut<PlayerNoAction>,
    input: Res<PlayerInputState>,
    map: Res<Map>,
    turn_order: Res<TurnOrder>,
    // TODO: do the actual move in a knock-on system, so everything is immutable except event launching
    mut player_query: Query<(&mut WorldPos,), With<Player>>,
    blocked: Res<BlockedTiles>,
    combats: Res<CombatStatsTiles>,
    mut player_map: ResMut<PlayerDistanceMap>,
    mut turn_events: EventWriter<EntityFinishedTurn>,
    mut moved_events: EventWriter<EntityMovedEvent>,
    mut attack_events: EventWriter<EntityMeleeAttacks>,
) {
    let entity = match turn_order.current_holder() {
        Some(entity) => entity,
        None => return,
    };

    let (mut wp,) = match player_query.get_mut(entity) {
        Ok(tup) => tup,
        // not the player's turn, so do nothing
        Err(_) => return,
    };

    if player_lock.0 {
        player_no_action.0 = true;
        return;
    }

    // This doesn't necessarily mean the player did something; it just prevents this system from
    // running more than once per frame
    player_lock.0 = true;

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
        turn_events.send(EntityFinishedTurn { entity });
    } else if new_wp != *wp {
        if let Some(defender) = combats.get_any(new_wp) {
            turn_events.send(EntityFinishedTurn { entity });
            attack_events.send(EntityMeleeAttacks {
                attacker: entity,
                defender,
            });
        } else if can_pass(new_wp, &*map, &*blocked) {
            turn_events.send(EntityFinishedTurn { entity });
            moved_events.send(EntityMovedEvent {
                entity,
                old_pos: *wp,
                new_pos: new_wp,
            });
            *wp = new_wp;

            // Note we don't want to block on "temporary" blocks, just walls; this has better
            // monster behavior resulting (piling vs random running around)
            // TODO: pull this out to a separate system that triggers on move
            let new_player_map = dijkstra::distance_dijkstra_map(&*map, [new_wp].iter(), |_| false);
            player_map.0 = new_player_map;
        }
    } else {
        player_no_action.0 = true;
    }
}

// TODO: make a macro to do this for everybody
fn update_blocked_map(
    q: Query<(), (With<WorldPos>, With<BlocksMovement>)>,
    mut blocked: ResMut<BlockedTiles>,
    mut events: EventReader<EntityMovedEvent>,
) {
    for event in events.iter() {
        let EntityMovedEvent {
            entity,
            old_pos,
            new_pos,
        } = *event;
        if let Ok(_) = q.get(entity) {
            blocked.update_entity(old_pos, new_pos, entity);
        }
    }
}

fn update_combat_stats_map(
    q: Query<(), (With<WorldPos>, With<CombatStats>)>,
    mut blocked: ResMut<CombatStatsTiles>,
    mut events: EventReader<EntityMovedEvent>,
) {
    for event in events.iter() {
        let EntityMovedEvent {
            entity,
            old_pos,
            new_pos,
        } = *event;
        if let Ok(_) = q.get(entity) {
            blocked.update_entity(old_pos, new_pos, entity);
        }
    }
}

fn next_turn(
    mut finished_turn_events: EventReader<EntityFinishedTurn>,
    mut turns: ResMut<TurnOrder>,
) {
    if let Some(_) = finished_turn_events.iter().next() {
        turns.end_turn();
    }
}

// TODO: make a macro to clear all these events
fn drain_turn_events(
    mut finished_turn_events: ResMut<Events<EntityFinishedTurn>>,
    mut move_events: ResMut<Events<EntityMovedEvent>>,
    mut combat_events: ResMut<Events<EntityMeleeAttacks>>,
    mut suffers: ResMut<Events<EntitySuffersDamage>>,
    mut died: ResMut<Events<EntityDies>>,
) {
    finished_turn_events.clear();
    move_events.clear();
    combat_events.clear();
    suffers.clear();
    died.clear();
}

fn can_pass(new_wp: WorldPos, map: &Map, blocked: &BlockedTiles) -> bool {
    map.passable(new_wp) && !blocked.0.has_any(new_wp)
}

pub fn death_system(mut deaths: EventReader<EntityDies>, mut commands: Commands) {
    for event in deaths.iter() {
        let EntityDies { entity } = *event;
        commands.entity(entity).despawn();
    }
}

// TODO make this another macro
pub fn remove_dead_from_maps(
    mut deaths: EventReader<EntityDies>,
    mut turns: ResMut<TurnOrder>,
    mut block_maps: ResMut<BlockedTiles>,
    mut cs_maps: ResMut<CombatStatsTiles>,
) {
    for event in deaths.iter() {
        let EntityDies { entity } = *event;
        turns.remove_from_turn_order(entity);
        block_maps.remove_entity_anywhere(entity);
        cs_maps.remove_entity_anywhere(entity);
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
    mut query_set: QuerySet<(
        QueryState<(Entity, &WorldPos), With<Player>>,
        // TODO: do the actual move in a knock-on system, so everything is immutable except event launching
        QueryState<(&Viewshed, &mut WorldPos), With<MonsterAI>>,
    )>,
    map: Res<Map>,
    blocked: Res<BlockedTiles>,
    player_map: Res<PlayerDistanceMap>,
    turns: Res<TurnOrder>,
    mut entity_turns: EventWriter<EntityFinishedTurn>,
    mut entity_moves: EventWriter<EntityMovedEvent>,
    mut entity_attacks: EventWriter<EntityMeleeAttacks>,
) {
    let entity = match turns.current_holder() {
        Some(entity) => entity,
        None => return,
    };

    // if monsters can't choose an action the game still moves on
    if query_set.q1().get(entity).is_ok() {
        entity_turns.send(EntityFinishedTurn { entity });
    } else {
        return;
    }

    let (player_entity, player_pos) = match query_set.q0().iter().next() {
        Some((entity, wp)) => (entity, *wp),
        // no player no action
        None => return,
    };

    query_set
        .q0()
        .get(player_entity)
        .expect("Thing should exist");

    match query_set.q1().get_mut(entity) {
        Ok((vs, mut wp)) => {
            if !vs.visible_tiles.contains(&player_pos) {
                // player is not visible; do nothing
                // don't chase them all over the map, only if they can be seen
                return;
            }

            if wp.dist(player_pos) <= 1 {
                entity_attacks.send(EntityMeleeAttacks {
                    attacker: entity,
                    defender: player_entity,
                });
            } else {
                let curr_dist = player_map.0.get(&*wp).copied().unwrap_or(i32::MAX);

                let mut new_wp = *wp;
                let mut best_dest = curr_dist;

                for adj_tile in map.adjacent(*wp).filter(|w| !blocked.has_any(*w)) {
                    let adj_dist = player_map.0.get(&adj_tile).copied().unwrap_or(i32::MAX);
                    if adj_dist < best_dest {
                        best_dest = adj_dist;
                        new_wp = adj_tile;
                    }
                }

                if new_wp != *wp {
                    entity_moves.send(EntityMovedEvent {
                        entity,
                        old_pos: *wp,
                        new_pos: new_wp,
                    });
                    *wp = new_wp;
                }
            }
        }
        Err(e) => unreachable!("We know the entity matches the query; error was {:?}", e),
    }
}

pub fn process_suffers_damage_event(
    mut cs_query: Query<&mut CombatStats>,
    name_query: Query<&EntityName>,
    mut damage_events: EventReader<EntitySuffersDamage>,
    mut death_events: EventWriter<EntityDies>,
) {
    for event in damage_events.iter() {
        let EntitySuffersDamage { entity, damage } = *event;

        match cs_query.get_mut(entity) {
            Ok(mut cs) => {
                cs.hp -= damage;

                let name = name_query
                    .get(entity)
                    .map(|n| n.0.as_str())
                    .unwrap_or("[unknown]");
                if cs.hp <= 0 {
                    println!("{} has died from the {} damage", name, damage);
                    death_events.send(EntityDies { entity });
                } else {
                    println!(
                        "{} has suffered {} damage and has {} health remaining",
                        name, damage, cs.hp
                    );
                }
            }
            Err(_) => {}
        }
    }
}

pub fn process_combat_event(
    mut events: EventReader<EntityMeleeAttacks>,
    mut damage_events: EventWriter<EntitySuffersDamage>,
    cs_query: Query<&CombatStats>,
    name_query: Query<&EntityName>,
) {
    for event in events.iter() {
        let EntityMeleeAttacks { attacker, defender } = *event;
        let attacker_cs: CombatStats = match cs_query.get(attacker) {
            Ok(cs) => *cs,
            Err(_) => continue,
        };
        let defender_cs: CombatStats = match cs_query.get(defender) {
            Ok(cs) => *cs,
            Err(_) => continue,
        };

        let inflicted: i32 = (attacker_cs.power - defender_cs.defense).max(0);
        let attacker_name: &str = name_query
            .get(attacker)
            .map(|name| name.0.as_str())
            .unwrap_or("[unknown]");
        let defender_name: &str = name_query
            .get(defender)
            .map(|name| name.0.as_str())
            .unwrap_or("[unknown]");

        println!(
            "{} attacks {} for {} damage",
            attacker_name, defender_name, inflicted
        );

        damage_events.send(EntitySuffersDamage {
            entity: defender,
            damage: inflicted,
        });
    }
}

pub fn update_fps_text(
    diagnostics: Res<Diagnostics>,
    kb_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Text, Option<&mut Visibility>), With<FpsTextBox>>,
) {
    let toggled: bool = kb_input.just_pressed(KeyCode::F);
    for (mut text, vis) in query.iter_mut() {
        if toggled {
            vis.map(|mut v| v.is_visible = !v.is_visible);
        }
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}
