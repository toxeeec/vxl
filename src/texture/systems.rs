use super::Textures;
use bevy::prelude::*;

pub(super) fn create_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image: Handle<Image> = asset_server.load("blocks.png");
    let texture_atlas = TextureAtlas::from_grid(image, Vec2 { x: 8.0, y: 8.0 }, 8, 8, None, None);
    let blocks = texture_atlases.add(texture_atlas);
    commands.insert_resource(Textures { blocks });
}
