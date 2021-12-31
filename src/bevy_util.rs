use bevy::prelude::*;

// todo customize color, size, font, etc.
pub fn make_text_bundle(sigil: char, asset_server: &Res<AssetServer>) -> Text2dBundle {
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
