use bevy::prelude::*;

#[derive(Resource, Debug)]
pub(crate) struct Textures {
    pub(crate) blocks: Handle<TextureAtlas>,
}
