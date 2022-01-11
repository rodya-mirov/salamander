use bevy::prelude::*;
use rand::Rng;

use crate::bevy_util::make_basic_sprite_bundle;
use crate::components::*;
use crate::map::*;
use crate::resources::*;

pub fn make_map(
    mut map_res: ResMut<Map>,
    mut events: ResMut<CallbackEvents>,
    mut commands: Commands,
    sheet: Res<BasicTilesAtlas>,
) {
    let (map, rooms) = make_new_map();

    let room = rooms[0];
    let (x, y) = room.center();

    commands
        .spawn()
        .insert(Player)
        .insert(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .insert(EntityName("Player".to_string()))
        .insert_bundle(make_basic_sprite_bundle(2, &sheet.0, Color::ALICE_BLUE))
        .insert(Viewshed::new())
        .insert(RequiresSeen)
        .insert(WantsTurnOrderAssignment)
        .insert(WantsMapIndexing)
        .insert(BlocksMovement)
        .insert(WorldPos { x, y })
        // TODO: coherent layering management system, not like this
        .insert(Transform::from_xyz(0.0, 0.0, 100.0));

    let mut rng = rand::thread_rng();

    let make_sprite = |kind| match kind {
        MonsterKind::KnifeOrc => make_basic_sprite_bundle(0, &sheet.0, Color::LIME_GREEN),
        MonsterKind::StrongOrc => make_basic_sprite_bundle(33, &sheet.0, Color::ORANGE_RED),
    };

    let mut idx = 0;
    let mut make_name = |kind| {
        let this_idx = idx;
        idx += 1;
        match kind {
            MonsterKind::KnifeOrc => EntityName(format!("Knife-wielding orc #{}", this_idx)),
            MonsterKind::StrongOrc => EntityName(format!("Orc #{}", this_idx)),
        }
    };

    let make_stats = |kind| match kind {
        MonsterKind::KnifeOrc => CombatStats {
            max_hp: 12,
            hp: 12,
            defense: 1,
            power: 4,
        },
        MonsterKind::StrongOrc => CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 2,
            power: 3,
        },
    };

    for room in rooms.iter().skip(1) {
        let (x, y) = room.center();

        let kind = match rng.gen_range(0..2) {
            0 => MonsterKind::KnifeOrc,
            1 => MonsterKind::StrongOrc,
            _ => unreachable!(),
        };

        commands
            .spawn()
            .insert(Viewshed::new())
            .insert(WorldPos { x, y })
            .insert(RequiresSeen)
            .insert(MonsterAI)
            .insert(BlocksMovement)
            .insert(WantsTurnOrderAssignment)
            .insert(WantsMapIndexing)
            .insert_bundle(make_sprite(kind))
            .insert(make_name(kind))
            .insert(make_stats(kind))
            .insert(Transform::from_xyz(0.0, 0.0, 40.0));
    }

    *map_res = map;

    events.send(MapChangedEvent);
}

pub fn setup_turn_counter(mut commands: Commands) {
    commands
        .spawn()
        .insert(EndOfTurnTrigger)
        .insert(WantsTurnOrderAssignment);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum MonsterKind {
    StrongOrc,
    KnifeOrc,
}

pub fn load_tileset(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let sheet_handle: Handle<_> = asset_server.load("tiles/basic_tiles.png");
    let texture_atlas = TextureAtlas::from_grid(sheet_handle, Vec2::new(32.0, 32.0), 16, 20);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(BasicTilesAtlas(texture_atlas_handle));
}

pub fn setup_stock_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            node: Default::default(),
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    right: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                            font_size: 20.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                alignment: Default::default(),
            },
            ..Default::default()
        })
        // false by default, can toggle with a system
        .insert(Visibility { is_visible: false })
        .insert(FpsTextBox);
}
