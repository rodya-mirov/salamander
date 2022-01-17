use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_startup_system(setup)
        .add_startup_system(turn_off_gravity)
        .add_system(player_input)
        .add_system(player_camera_control)
        .add_system(animate_sprite_system)
        .add_system(follow_player)
        .add_system(cancel_rotation)
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
const PLAYER_PIXELS_PER_SEC: f32 = 100128.0;

// scale is multiplicative, so this is applied additively to the log of the camera scale
// so 1/e is "doubles in one second", 2/e is "quadruples in one second" and so on.
const CAMERA_ZOOM_SPEED_PER_SEC: f32 = 1.0;

fn player_input(
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut RigidBodyVelocityComponent,
            &RigidBodyMassPropsComponent,
        ),
        With<Player>,
    >,
) {
    let dist = time.delta().as_secs_f32() * PLAYER_PIXELS_PER_SEC;
    let mut desired: Vec2 = Vec2::new(0.0, 0.0);

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

    if desired != Vec2::default() {
        desired /= desired.length();
        desired *= dist;
    }

    for (mut rb_vel, rb_mprops) in query.iter_mut() {
        let rb_vel: &mut RigidBodyVelocityComponent = &mut *rb_vel;
        let rb_mprops: &RigidBodyMassPropsComponent = &*rb_mprops;
        rb_vel.apply_impulse(rb_mprops, desired.into());
    }
}

fn follow_player(
    time: Res<Time>,
    mut qs: QuerySet<(
        QueryState<&Transform, With<Player>>,
        QueryState<&mut Transform, With<PlayerCamera>>,
    )>,
) {
    let dist = time.delta().as_secs_f32() * 500000.0; // PLAYER_PIXELS_PER_SEC;

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

fn cancel_rotation(mut query: Query<&mut RigidBodyPositionComponent, With<NoRotate>>) {
    for mut rbpc in query.iter_mut() {
        rbpc.position.rotation = Default::default();
    }
}

#[derive(Component)]
struct NoRotate;

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

    for x in -10..=10 {
        let y = 10;

        let pos = Vec2::new((x as f32) * 16.0, (y as f32) * 16.0);
        setup_brick(&mut commands, &*asset_server, &mut *texture_atlases, pos);
    }

    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.75;

    // setup camera
    commands.spawn_bundle(camera).insert(PlayerCamera);
}

fn turn_off_gravity(mut config: ResMut<RapierConfiguration>) {
    config.gravity = Vector::<f32>::default();
    config.scale = 1.0;
}

fn setup_player_character(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) {
    let texture_handle = asset_server.load("sprites/gabe-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let pos = Vec2::new(32.0, 32.0);

    let mut player = commands.spawn();

    player
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(pos.x, pos.y, 2.0),
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(NoRotate)
        .insert(Player);

    setup_rigid_body(&mut player, pos.x, pos.y, 8.0);
}

fn setup_rigid_body(spawner: &mut bevy_ecs::system::EntityCommands, x: f32, y: f32, rad: f32) {
    let pos = Vec2::new(x, y);

    spawner
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic.into(),
            position: pos.into(),
            damping: RigidBodyDamping {
                linear_damping: 5.0,
                angular_damping: 57891.0,
            }
            .into(),
            ccd: RigidBodyCcd {
                ccd_enabled: true,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(rad).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete);
}

fn setup_square_static_body(spawner: &mut bevy_ecs::system::EntityCommands, pos: Vec2, rad: f32) {
    spawner
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static.into(),
            position: pos.into(),
            damping: RigidBodyDamping {
                linear_damping: 5.0,
                angular_damping: 57891.0,
            }
            .into(),
            ccd: RigidBodyCcd {
                ccd_enabled: true,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(rad).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete);
}

fn setup_brick(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    pos: Vec2,
) {
    let texture_handle = asset_server.load("tiles/Level.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 6, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut npc = commands.spawn();

    npc.insert_bundle(SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        transform: Transform::from_xyz(pos.x, pos.y, 0.0),
        ..Default::default()
    })
    .insert(NoRotate);

    setup_square_static_body(&mut npc, pos, 8.0);
}

fn setup_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
) {
    let texture_handle = asset_server.load("sprites/mani-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut npc = commands.spawn();

    let pos = Vec2::new(0.0, 0.0);

    npc.insert_bundle(SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        transform: Transform::from_xyz(pos.x, pos.y, 1.0),
        ..Default::default()
    })
    .insert(NoRotate)
    .insert(Timer::from_seconds(0.1, true));

    setup_rigid_body(&mut npc, pos.x, pos.y, 8.0);
}
