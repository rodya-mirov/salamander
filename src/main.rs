use bevy::prelude::*;

pub(crate) mod components;
pub(crate) mod resources;
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
        use resources::*;

        app.insert_resource(PlayerInputState::default())
            // setup systems
            .add_startup_system(setup_systems::world_setup.system())
            .add_startup_system(camera_setup.system())
            // input systems
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
                    .after("player actions"),
            )
            // graphical systems
            .add_system_set(
                SystemSet::new()
                    .label("graphics updates")
                    .after("npc actions")
                    .with_system(running_systems::aim_camera.system())
                    .with_system(running_systems::world_pos_to_visual_system.system()),
            );
    }
}

pub fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Salamander".to_string(),
            width: components::TILE_SIZE * components::MAP_WIDTH_TILES as f32,
            height: components::TILE_SIZE * components::MAP_HEIGHT_TILES as f32,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MapPlugin)
        .run();
}
