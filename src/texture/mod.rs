mod resources;
mod systems;

use bevy::prelude::*;
pub(crate) use resources::Textures;
use systems::create_textures;

#[derive(Debug)]
pub(crate) struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_textures);
    }
}
