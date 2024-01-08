mod resources;
mod systems;

pub(crate) use resources::CenterOffset;

use crate::{block::BlockId, position::Offset, texture::ChunkMaterial};
use bevy::{
    app::{App, Plugin},
    prelude::*,
    tasks::Task,
};
use resources::Chunks;
use std::sync::Arc;
use systems::{
    despawn_distant_chunks, handle_meshing_tasks, handle_spawning_tasks, mesh_chunks, spawn_chunks,
    ChunkSpawningTask,
};

#[derive(Component, Debug)]
struct Chunk;

#[derive(Component, Debug)]
struct Dirty;

#[derive(Bundle)]
struct ChunkBundle {
    mmb: MaterialMeshBundle<ChunkMaterial>,
    chunk: Chunk,
    task: ChunkSpawningTask,
}

impl ChunkBundle {
    fn new(
        offset: Offset,
        material: Handle<ChunkMaterial>,
        task: Task<(Offset, Arc<[BlockId]>)>,
    ) -> Self {
        Self {
            mmb: MaterialMeshBundle {
                transform: offset.into(),
                material,
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            task: ChunkSpawningTask(task),
            chunk: Chunk,
        }
    }
}

#[derive(Debug)]
pub(super) struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>().add_systems(
            Update,
            (
                (
                    (handle_spawning_tasks, despawn_distant_chunks).chain(),
                    (mesh_chunks, spawn_chunks),
                )
                    .chain(),
                handle_meshing_tasks,
            ),
        );
    }
}
