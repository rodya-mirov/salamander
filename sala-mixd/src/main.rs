use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_startup_system(setup)
        .add_system(player_input)
        .add_system(player_camera_control)
        .add_system(animate_sprite_system)
        .add_system(follow_player)
        .run();
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

// pixels per second of movement; this is in world space, not camera space
const PLAYER_PIXELS_PER_SEC: f32 = 128.0;

// scale is multiplicative, so this is applied additively to the log of the camera scale
// so 1/e is "doubles in one second", 2/e is "quadruples in one second" and so on
// must be positive
const CAMERA_ZOOM_SPEED_PER_SEC: f32 = 1.0;

fn player_input(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let dist = time.delta().as_secs_f32() * PLAYER_PIXELS_PER_SEC;
    let mut desired: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    if kb.pressed(KeyCode::Left) {
        desired.x -= dist;
    }
    if kb.pressed(KeyCode::Right) {
        desired.x += dist;
    }
    if kb.pressed(KeyCode::Up) {
        desired.y += dist;
    }
    if kb.pressed(KeyCode::Down) {
        desired.y -= dist;
    }

    if desired == Vec3::default() {
        return;
    }

    desired /= desired.length();
    desired *= dist;

    for mut transform in query.iter_mut() {
        transform.translation += desired;
    }
}

fn follow_player(
    time: Res<Time>,
    mut qs: QuerySet<(
        QueryState<&Transform, With<Player>>,
        QueryState<&mut Transform, With<PlayerCamera>>,
    )>,
) {
    let dist = time.delta().as_secs_f32() * PLAYER_PIXELS_PER_SEC;

    let player_pos: Vec3 = match qs.q0().iter().next() {
        Some(t) => t.translation,
        None => return,
    };

    for mut camera_transform in qs.q1().iter_mut() {
        let diff: Vec3 = player_pos - camera_transform.translation;
        let length = diff.length();
        if length < dist {
            camera_transform.translation = player_pos;
        } else {
            let unit = diff / length;
            let dest = unit * dist;
            camera_transform.translation += dest;
        }
    }
}

fn player_camera_control(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut OrthographicProjection, With<PlayerCamera>>,
) {
    let dist = CAMERA_ZOOM_SPEED_PER_SEC * time.delta().as_secs_f32();

    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();

        if kb.pressed(KeyCode::PageUp) {
            log_scale -= dist;
        }
        if kb.pressed(KeyCode::PageDown) {
            log_scale += dist;
        }

        projection.scale = log_scale.exp();
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCamera;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    setup_player_character(&mut commands, &*asset_server, &mut *texture_atlases);
    setup_npc(&mut commands, &*asset_server, &mut *texture_atlases);

    // setup camera
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(PlayerCamera);
}

fn setup_player_character(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) {
    let texture_handle = asset_server.load("sprites/gabe-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Player);
}

fn setup_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) {
    let texture_handle = asset_server.load("sprites/mani-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true));
}
