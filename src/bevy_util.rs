use bevy::prelude::*;

pub fn make_basic_sprite_bundle(
    index: usize,
    sheet_handle: &Handle<TextureAtlas>,
    color: Color,
) -> SpriteSheetBundle {
    SpriteSheetBundle {
        texture_atlas: sheet_handle.clone(),
        sprite: TextureAtlasSprite {
            index,
            color,
            ..Default::default()
        },
        ..Default::default()
    }
}
