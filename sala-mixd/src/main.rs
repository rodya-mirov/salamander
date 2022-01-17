use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
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
const PLAYER_PIXELS_PER_SEC: f32 = 128.0;

// scale is multiplicative, so this is applied additively to the log of the camera scale
// so 1/e is "doubles in one second", 2/e is "quadruples in one second" and so on.
const CAMERA_ZOOM_SPEED_PER_SEC: f32 = 1.0;

fn player_input(
    kb: Res<Input<KeyCode>>,
    mut query: Query<&mut RigidBodyVelocityComponent, With<Player>>,
) {
    // let dist = time.delta().as_secs_f32() * PLAYER_PIXELS_PER_SEC;
    let dist = PLAYER_PIXELS_PER_SEC;
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

    for mut rb_vel in query.iter_mut() {
        let vel: &mut RigidBodyVelocityComponent = &mut *rb_vel;
        vel.linvel = desired.into();
        vel.angvel = 0.0;
    }
}

fn follow_player(
    mut qs: QuerySet<(
        QueryState<&Transform, With<Player>>,
        QueryState<&mut Transform, With<PlayerCamera>>,
    )>,
) {
    let player_pos: Vec3 = match qs.q0().iter().next() {
        Some(t) => t.translation,
        None => return,
    };

    for mut camera_transform in qs.q1().iter_mut() {
        camera_transform.translation = player_pos;
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

// TODO: make sure this is actually happening after the rapier pass, so we don't have frame lag
fn cancel_rotation(
    mut query: Query<
        (
            Option<&mut RigidBodyPositionComponent>,
            Option<&mut RigidBodyVelocityComponent>,
        ),
        With<NoRotate>,
    >,
) {
    for (rbpc, rbvc) in query.iter_mut() {
        rbpc.map(|mut rbpc| rbpc.position.rotation = Default::default());
        rbvc.map(|mut rbvc| rbvc.angvel = 0.0);
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    setup_player_character(
        &mut commands,
        &*asset_server,
        &mut *texture_atlases,
        &mut *meshes,
        &mut *materials,
    );
    setup_npc(
        &mut commands,
        &*asset_server,
        &mut *texture_atlases,
        &mut *meshes,
        &mut *materials,
    );

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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let texture_handle = asset_server.load("sprites/gabe-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut player = commands.spawn();

    player.insert(NoRotate).insert(Player);

    let config = SpriteBodyConfig {
        pos: Vec2::new(32.0, 32.0),
        collider_radius: 8.0,
        sprite_offset: Vec2::new(0.0, 8.0),
        texture_atlas_handle,
        shadow_mesh: meshes
            .add(Mesh::from(bevy::prelude::shape::Quad::new(Vec2::new(
                16.0, 16.0,
            ))))
            .into(),
        shadow_color: materials.add(ColorMaterial::from(Color::BLACK)),
    };

    setup_sprited_body(&mut player, config);
}

fn setup_npc(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let texture_handle = asset_server.load("sprites/mani-idle-run.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut npc = commands.spawn();

    npc.insert(NoRotate);

    let config = SpriteBodyConfig {
        pos: Vec2::new(0.0, 0.0),
        collider_radius: 8.0,
        sprite_offset: Vec2::new(0.0, 8.0),
        texture_atlas_handle,
        shadow_mesh: meshes
            .add(Mesh::from(bevy::prelude::shape::Quad::new(Vec2::new(
                16.0, 16.0,
            ))))
            .into(),
        shadow_color: materials.add(ColorMaterial::from(Color::BLACK)),
    };

    setup_sprited_body(&mut npc, config);
}

#[derive(Clone)]
struct SpriteBodyConfig {
    // position in world space
    pos: Vec2,
    // collider will be a circle with this radius, centered at the pos
    collider_radius: f32,
    // offset above the shadow
    sprite_offset: Vec2,
    // handle to the main sprite's texture
    texture_atlas_handle: Handle<TextureAtlas>,
    // shadow stuff
    shadow_mesh: Mesh2dHandle,
    shadow_color: Handle<ColorMaterial>,
}

fn setup_sprited_body(
    spawner: &mut bevy_ecs::system::EntityCommands,
    sprite_config: SpriteBodyConfig,
) {
    let pos = sprite_config.pos;

    spawner
        .insert_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Dynamic.into(),
            position: pos.into(),
            damping: RigidBodyDamping {
                linear_damping: 5.0,
                angular_damping: 1.0,
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
            shape: ColliderShape::ball(sprite_config.collider_radius).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete);

    spawner.with_children(|s: &mut ChildBuilder| {
        let mut shadow = s.spawn();
        // TODO: make shadow visible
        shadow.insert_bundle(MaterialMesh2dBundle {
            mesh: sprite_config.shadow_mesh,
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
            material: sprite_config.shadow_color,
            ..Default::default()
        });

        let mut main_sprite = s.spawn();
        main_sprite
            .insert_bundle(SpriteSheetBundle {
                texture_atlas: sprite_config.texture_atlas_handle,
                transform: Transform::from_xyz(
                    sprite_config.sprite_offset.x,
                    sprite_config.sprite_offset.y,
                    -1.0,
                ),
                ..Default::default()
            })
            .insert(Timer::from_seconds(0.1, true));
    });
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
        transform: Transform::from_xyz(pos.x, pos.y, -10.0),
        ..Default::default()
    })
    .insert(NoRotate);

    setup_square_static_body(&mut npc, pos, 8.0);
}
