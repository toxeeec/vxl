mod resources;
mod systems;

pub(crate) use resources::CenterOffset;

use crate::player::move_player;
use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use resources::Chunks;
use systems::{handle_meshing_tasks, mesh_chunks, reorder_chunks, spawn_chunks, unload_distant_chunks};

#[derive(Component, Debug)]
struct Chunk;

#[derive(Component, Debug)]
struct Dirty;

#[derive(Debug)]
pub(super) struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>()
            .add_systems(
                Update,
                (unload_distant_chunks, reorder_chunks, spawn_chunks)
                    .chain()
                    .after(move_player),
            )
            .add_systems(PostUpdate, (mesh_chunks, handle_meshing_tasks).chain());
    }
}
