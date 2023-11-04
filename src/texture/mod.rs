mod resources;
mod systems;
pub(crate) mod uv;

pub(crate) use resources::Textures;

use bevy::prelude::*;
use systems::create_textures;

#[derive(Debug)]
pub(crate) struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_textures);
    }
}
