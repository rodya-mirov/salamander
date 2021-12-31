use bevy::prelude::*;

use crate::components::{Player, WorldPos};

pub fn world_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(make_text_bundle('@', &asset_server))
        .insert(WorldPos { x: 26, y: 0 })
        .insert(Transform::default());
}

// todo customize color, size, font, etc.
fn make_text_bundle(sigil: char, asset_server: &Res<AssetServer>) -> Text2dBundle {
    let section = TextSection {
        value: format!("{}", sigil),
        style: TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 32.0,
            color: Color::rgb(0.5, 1.0, 0.5),
        },
    };

    Text2dBundle {
        text: Text {
            sections: vec![section],
            alignment: TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        },
        ..Default::default()
    }
}
