mod bundles;
mod materials;
mod offset;
mod resources;
mod systems;

pub(crate) use materials::ChunkMaterial;

use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use systems::{mesh_chunks, spawn_chunks};

#[derive(Component, Debug)]
struct Chunk;

#[derive(Component, Debug)]
struct Dirty;

#[derive(Debug)]
pub(crate) struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_systems(Startup, spawn_chunks)
            .add_systems(Update, mesh_chunks);
    }
}
