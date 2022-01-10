use bevy::ecs::schedule::IntoSystemDescriptor;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

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

    commands.spawn_bundle(UiCameraBundle::default());
}

trait AppExtension {
    fn add_sequential_system<Params>(
        &mut self,
        idx: &mut usize,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self;
}

impl AppExtension for App {
    fn add_sequential_system<Params>(
        &mut self,
        idx: &mut usize,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        let mut ss = SystemSet::new();

        if *idx > 0 {
            let after: &'static str =
                Box::leak(format!("CustomSystemSet{}", *idx).into_boxed_str());
            ss = ss.after(after);
        }

        *idx += 1;

        let label: &'static str = Box::leak(format!("CustomSystemSet{}", *idx).into_boxed_str());

        ss = ss.label(label).with_system(system);

        self.add_system_set(ss)
    }
}

impl AppExtension for SystemStage {
    fn add_sequential_system<Params>(
        &mut self,
        idx: &mut usize,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        let mut ss = SystemSet::new();

        if *idx > 0 {
            let after: &'static str =
                Box::leak(format!("CustomSystemSet{}", *idx).into_boxed_str());
            ss = ss.after(after);
        }

        *idx += 1;

        let label: &'static str = Box::leak(format!("CustomSystemSet{}", *idx).into_boxed_str());

        ss = ss.label(label).with_system(system);

        self.add_system_set(ss)
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        const ASSET_LOADING: &str = "load assets";
        const WORLD_SETUP: &str = "setup map";

        use components::*;
        use map::*;
        use resources::*;

        app.insert_resource(PlayerInputState::default())
            .insert_resource(Map::default())
            .insert_resource(PlayerMovedInFrame::default())
            .insert_resource(PlayerNoAction::default())
            .insert_resource(BlockedTiles::default())
            .insert_resource(CombatStatsTiles::default())
            .insert_resource(PlayerDistanceMap::default())
            .insert_resource(TurnOrder::default())
            .add_event::<MapChangedEvent>()
            .add_event::<VisibilityChangedEvent>()
            .add_event::<EntityMovedEvent>()
            .add_event::<EntityMeleeAttacks>()
            .add_event::<EntityFinishedTurn>()
            .add_event::<EntitySuffersDamage>()
            .add_event::<EntityDies>()
            // asset loading
            .add_startup_stage(ASSET_LOADING, SystemStage::single_threaded())
            .add_startup_system_to_stage(ASSET_LOADING, setup_systems::load_tileset)
            // setup systems
            .add_startup_stage_after(ASSET_LOADING, WORLD_SETUP, SystemStage::single_threaded())
            .add_startup_system_to_stage(WORLD_SETUP, setup_systems::make_map)
            .add_startup_system_to_stage(WORLD_SETUP, camera_setup)
            .add_startup_system(setup_systems::setup_stock_text)
            // input systems
            // TODO: remove this once we have real UI around this
            .add_system(bevy::input::system::exit_on_esc_system)
            .add_system(running_systems::update_fps_text)
            // i guess this is sloppy use of bevy but damn it i want my callbacks to be processed in one frame
            .add_system(running_systems::world_tick.exclusive_system())
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

    App::new()
        .insert_resource(WindowDescriptor {
            title: "".to_string(),
            width: TILE_SIZE * 31.0,
            height: TILE_SIZE * 25.0,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(MapPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
