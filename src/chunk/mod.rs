mod components;
mod direction;
mod systems;

use bevy::{
    app::{App, Plugin},
    prelude::{Startup, Update},
};
use systems::{mesh_chunks, spawn_chunk};

pub(crate) const CHUNK_WIDTH: usize = 16;
pub(crate) const CHUNK_HEIGHT: usize = 256;
pub(crate) const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
pub(crate) const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;

#[derive(Debug)]
pub(crate) struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chunk)
            .add_systems(Update, mesh_chunks);
    }
}
