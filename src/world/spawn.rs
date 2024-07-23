use std::sync::Arc;

use bevy::{
    math::bounding::Aabb2d,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelExtend, ParallelIterator};

use crate::{
    player::PlayerChunkMoveEvent,
    settings::RENDER_DISTANCE,
    texture::{ChunkMaterial, ChunkTexture},
};

use super::{
    mesh::ChunkMeshingTasks, Chunk, ChunkEntities, Chunks, DirtyChunks, Noise, WorldPlugin,
    WorldgenParams,
};

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkSpawningTasks(HashMap<IVec2, Task<Chunk>>);

impl WorldPlugin {
    pub(super) fn generate_world(
        mut commands: Commands,
        noise: Res<Noise>,
        params: Res<WorldgenParams>,
        texture: Res<ChunkTexture>,
        mut chunks: ResMut<Chunks>,
        mut entities: ResMut<ChunkEntities>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let noise = noise.clone();
        let params = params.clone();

        chunks.0.par_extend(
            renderable_chunks(IVec2::ZERO)
                .par_bridge()
                .map(|offset| (offset, Arc::new(Chunk::generate(offset, &noise, &params)))),
        );

        entities.0.extend(
            chunks
                .0
                .par_iter()
                .map(|(&offset, chunk)| (offset, chunk.get_mesh(&chunks.get_neighbors(offset))))
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(offset, mesh)| {
                    (
                        offset,
                        commands
                            .spawn(MaterialMeshBundle {
                                material: materials
                                    .add(ChunkMaterial::new(offset, texture.0.clone())),
                                mesh: meshes.add(mesh),
                                ..Default::default()
                            })
                            .id(),
                    )
                }),
        );
    }

    pub(super) fn spawn_chunks(
        mut events: EventReader<PlayerChunkMoveEvent>,
        noise: Res<Noise>,
        entities: Res<ChunkEntities>,
        params: Res<WorldgenParams>,
        mut tasks: ResMut<ChunkSpawningTasks>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();
        let noise = Arc::new(noise.clone());

        for ev in events.read() {
            for offset in renderable_chunks(ev.new_offset) {
                if entities.0.contains_key(&offset) {
                    continue;
                }
                let noise = noise.clone();
                let params = params.clone();
                let task =
                    thread_pool.spawn(async move { Chunk::generate(offset, &noise, &params) });
                tasks.0.insert(offset, task);
            }
        }
    }

    pub(super) fn despawn_chunks(
        mut chunk_move_events: EventReader<PlayerChunkMoveEvent>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut spawning_tasks: ResMut<ChunkSpawningTasks>,
        mut meshing_tasks: ResMut<ChunkMeshingTasks>,
    ) {
        for &PlayerChunkMoveEvent { new_offset } in chunk_move_events.read() {
            let player_offset = IVec2::new(new_offset.x, new_offset.y);
            let mut to_remove = Vec::new();

            for offset in chunks.0.keys() {
                let aabb = Aabb2d::new(offset.as_vec2(), Vec2::splat(0.5));
                let distance = player_offset
                    .as_vec2()
                    .distance(aabb.closest_point(player_offset.as_vec2()));

                if distance > RENDER_DISTANCE as f32 {
                    to_remove.push(*offset);
                }
            }

            for offset in &to_remove {
                chunks.0.remove(offset);
                spawning_tasks.0.remove(offset);
                meshing_tasks.0.remove(offset);
                dirty.insert(*offset);
            }
        }
    }

    pub(super) fn sync_dirty_chunks(chunks: Res<Chunks>, mut dirty: ResMut<DirtyChunks>) {
        if !chunks.is_changed() {
            return;
        }
        dirty.0.retain(|offset| chunks.0.contains_key(offset));
    }

    pub(super) fn sync_chunk_entities(
        mut commands: Commands,
        chunks: Res<Chunks>,
        texture: Res<ChunkTexture>,
        mut entities: ResMut<ChunkEntities>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
    ) {
        if !chunks.is_changed() {
            return;
        }

        entities.0.retain(|offset, entity| {
            if !chunks.0.contains_key(offset) {
                commands.entity(*entity).despawn();
                false
            } else {
                true
            }
        });

        for offset in chunks.0.keys() {
            if entities.0.contains_key(offset) {
                continue;
            }

            entities.0.insert(
                *offset,
                commands
                    .spawn(MaterialMeshBundle {
                        material: materials.add(ChunkMaterial::new(*offset, texture.0.clone())),
                        ..Default::default()
                    })
                    .id(),
            );
        }
    }

    pub(super) fn handle_spawning_tasks(
        mut tasks: ResMut<ChunkSpawningTasks>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
    ) {
        tasks.0.retain(|&offset, task| {
            if let Some(chunk) = block_on(future::poll_once(task)) {
                chunks.0.insert(offset, chunk.into());
                dirty.insert(offset);
                false
            } else {
                true
            }
        });
    }
}

fn renderable_chunks(player_offset: IVec2) -> impl Iterator<Item = IVec2> {
    let player_offset = IVec2::new(player_offset.x, player_offset.y);
    let iter_x =
        (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32).map(move |x| x + player_offset.x);
    let iter_z =
        (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32).map(move |z| z + player_offset.y);

    iter_x.cartesian_product(iter_z).filter_map(move |(x, z)| {
        let offset = IVec2::new(x, z);
        let aabb = Aabb2d::new(offset.as_vec2(), Vec2::splat(0.5));
        let distance = player_offset
            .as_vec2()
            .distance(aabb.closest_point(player_offset.as_vec2()));

        if distance <= RENDER_DISTANCE as f32 {
            Some(offset)
        } else {
            None
        }
    })
}
