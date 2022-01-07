use bevy::prelude::*;

mod bevy_util;

pub(crate) mod components;
pub(crate) mod resources;

mod map;

mod running_systems;
mod setup_systems;

struct MapPlugin;

fn camera_setup(mut commands: Commands) {
    use components::*;

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PlayerCamera);
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        const ASSET_LOADING: &str = "load assets";
        const WORLD_SETUP: &str = "setup map";

        use components::*;
        use map::*;
        use resources::*;

        app.insert_resource(PlayerInputState::default())
            .insert_resource(Map::default())
            .add_event::<MapChangedEvent>()
            .add_event::<VisibilityChangedEvent>()
            .add_event::<PlayerMovedEvent>()
            // asset loading
            .add_startup_stage(ASSET_LOADING, SystemStage::single_threaded())
            .add_startup_system_to_stage(ASSET_LOADING, setup_systems::load_tileset.system())
            // setup systems
            .add_startup_stage_after(ASSET_LOADING, WORLD_SETUP, SystemStage::single_threaded())
            .add_startup_system_to_stage(WORLD_SETUP, setup_systems::make_map.system())
            .add_startup_system_to_stage(WORLD_SETUP, camera_setup.system())
            // input systems
            // TODO: remove this once we have real UI around this
            .add_system(bevy::input::system::exit_on_esc_system.system())
            .add_system_set(
                SystemSet::new()
                    .label("get input")
                    .with_system(running_systems::get_player_input.system()),
            )
            .add_system_set(
                SystemSet::new()
                    .label("player actions")
                    .after("get input")
                    .with_system(running_systems::handle_input.system()),
            )
            // ai systems
            .add_system_set(
                SystemSet::new()
                    .label("npc actions")
                    .after("player actions")
                    .with_system(running_systems::monster_ai.system()),
            )
            // "consequences of player / npc actions" systems
            .add_system(
                running_systems::compute_viewsheds
                    .system()
                    .label("visibility")
                    .label("compute visibility")
                    .after("player actions")
                    .after("npc actions"),
            )
            .add_system(
                running_systems::update_map_visibility
                    .system()
                    .label("map visibility")
                    .after("compute visibility"),
            )
            // various cleanup actions which make sure certain visual systems are represented
            // TODO BUG: there is a screen flash every time I move, from the visual tiles getting wiped and not being rebuilt until the next turn
            .add_system_set(
                SystemSet::new()
                    .label("cleanup")
                    .after("map visibility")
                    .with_system(running_systems::rebuild_visual_tiles.system()),
            )
            // graphical systems; note they're in a separate stage so that commands will be issued correctly
            .add_stage_after(
                CoreStage::Update,
                "rebuild graphics",
                SystemStage::single_threaded(),
            )
            .add_system_set_to_stage(
                "rebuild graphics",
                SystemSet::new()
                    .with_system(running_systems::aim_camera.system())
                    .with_system(running_systems::hide_unseen_things.system())
                    .with_system(running_systems::world_pos_to_visual_system.system()),
            );
    }
}

pub fn main() {
    use map::TILE_SIZE;

    App::build()
        .insert_resource(WindowDescriptor {
            title: "".to_string(),
            width: TILE_SIZE * 31.0,
            height: TILE_SIZE * 25.0,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MapPlugin)
        .run();
}
