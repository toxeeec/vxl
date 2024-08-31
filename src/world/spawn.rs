use std::sync::Arc;

use bevy::{
    math::bounding::Aabb2d,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::{HashMap, HashSet},
};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use crate::{
    materials::ChunkMaterial, player::PlayerChunkMoveEvent, settings::RENDER_DISTANCE,
    textures::BlocksTexture,
};

use super::{
    db::Db, mesh::ChunkMeshingTasks, Chunk, ChunkEntities, Chunks, DirtyChunks, Noise, WorldPlugin,
    WorldgenParams,
};

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkSpawningTasks(HashMap<IVec2, Task<Chunk>>);

impl WorldPlugin {
    pub(super) fn generate_world(
        mut commands: Commands,
        noise: Res<Noise>,
        params: Res<WorldgenParams>,
        texture: Res<BlocksTexture>,
        db: Res<Db>,
        mut chunks: ResMut<Chunks>,
        mut entities: ResMut<ChunkEntities>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let noise = noise.clone();
        let params = params.clone();

        #[cfg(debug_assertions)]
        let radius = RENDER_DISTANCE;
        #[cfg(not(debug_assertions))]
        let radius = RENDER_DISTANCE * 2;

        let world: Vec<_> = chunks_around(IVec2::ZERO, radius)
            .par_bridge()
            .map(|offset| (offset, Arc::new(Chunk::generate(offset, &noise, &params))))
            .collect();

        chunks.0.extend(world.iter().filter_map(|(offset, chunk)| {
            if distance_between(IVec2::ZERO, *offset) <= RENDER_DISTANCE as f32 {
                Some((*offset, chunk.clone()))
            } else {
                None
            }
        }));

        block_on(db.insert_chunks(world));

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
                                material: materials.add(ChunkMaterial::new(offset, &texture.0)),
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
        db: Res<Db>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut tasks: ResMut<ChunkSpawningTasks>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();
        let noise = Arc::new(noise.clone());

        for ev in events.read() {
            let offsets = chunks_around(ev.new_offset, RENDER_DISTANCE)
                .filter(|offset| !entities.0.contains_key(offset));

            let offsets = block_on(async {
                let mut offsets = HashSet::from_iter(offsets);
                for row in db.get_chunks(offsets.iter()).await {
                    let offset = IVec2::new(row.x, row.z);
                    chunks.0.insert(offset, Arc::new(row.blocks));
                    dirty.0.insert(offset);
                    offsets.remove(&IVec2::new(row.x, row.z));
                }
                offsets
            });

            for offset in offsets {
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

            for &offset in chunks.0.keys() {
                let distance = distance_between(player_offset, offset);
                if distance > RENDER_DISTANCE as f32 {
                    to_remove.push(offset);
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
        texture: Res<BlocksTexture>,
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
                        material: materials.add(ChunkMaterial::new(*offset, &texture.0)),
                        ..Default::default()
                    })
                    .id(),
            );
        }
    }

    pub(super) fn handle_spawning_tasks(
        db: Res<Db>,
        mut tasks: ResMut<ChunkSpawningTasks>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
    ) {
        let mut spawned_chunks = Vec::new();

        tasks.0.retain(|&offset, task| {
            if let Some(chunk) = block_on(future::poll_once(task)) {
                let chunk = Arc::new(chunk);
                chunks.0.insert(offset, chunk.clone());
                dirty.insert(offset);
                spawned_chunks.push((offset, chunk));
                false
            } else {
                true
            }
        });

        if spawned_chunks.is_empty() {
            return;
        }

        block_on(db.insert_chunks(spawned_chunks));
    }
}

fn chunks_around(origin: IVec2, radius: i32) -> impl Iterator<Item = IVec2> {
    let iter_x = (-radius..=radius).map(move |x| x + origin.x);
    let iter_z = (-radius..=radius).map(move |z| z + origin.y);

    iter_x.cartesian_product(iter_z).filter_map(move |(x, z)| {
        let offset = IVec2::new(x, z);
        let distance = distance_between(origin, offset);
        if distance <= radius as f32 {
            Some(offset)
        } else {
            None
        }
    })
}

fn distance_between(a: IVec2, b: IVec2) -> f32 {
    let aabb = Aabb2d::new(b.as_vec2(), Vec2::splat(0.5));
    a.as_vec2().distance(aabb.closest_point(a.as_vec2()))
}
