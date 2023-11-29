mod resources;
mod systems;

pub(crate) use resources::CenterOffset;

use crate::{offset::Offset, player::player_movement, settings::RENDER_DISTANCE};
use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use resources::Chunks;
use systems::{mesh_chunks, reorder_chunks, spawn_chunks, unload_distant_chunks, update_offsets};

#[derive(Component, Debug)]
struct Chunk;

#[derive(Component, Debug)]
struct Dirty;

#[derive(Debug)]
pub(super) struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>()
            .add_systems(PreUpdate, unload_distant_chunks)
            .add_systems(
                Update,
                (
                    update_offsets,
                    reorder_chunks,
                    spawn_chunks.after(reorder_chunks),
                    mesh_chunks.after(spawn_chunks),
                )
                    .before(player_movement),
            );
    }
}

pub(super) fn chunk_in_bounds(transform: Transform, center_offset: Offset) -> bool {
    let dist = (Offset::from(transform).0 - center_offset.0).abs();
    dist.x <= RENDER_DISTANCE && dist.y <= RENDER_DISTANCE
}
