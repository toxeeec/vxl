mod systems;

use crate::block::Block;
use crate::direction::Direction;
use bevy::prelude::*;
use systems::create_textures;

#[derive(Debug)]
pub(super) struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_textures);
    }
}

#[derive(Resource, Debug)]
pub(crate) struct Textures {
    pub(crate) blocks: Handle<TextureAtlas>,
}

pub(crate) fn atlas_uvs(atlas: &TextureAtlas, block: Block, dir: Direction) -> [[f32; 2]; 4] {
    let area = atlas.textures[block.texture_id(dir)];
    let Vec2 { x: min_u, y: min_v } = (area.min + area.size() / 2.0 / atlas.size) / atlas.size;
    let Vec2 { x: max_u, y: max_v } = (area.max - area.size() / 2.0 / atlas.size) / atlas.size;

    [
        [max_u, min_v], // top-right
        [min_u, min_v], // top-left
        [min_u, max_v], // bottom-left
        [max_u, max_v], // bottom-right
    ]
}
