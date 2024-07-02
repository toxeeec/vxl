use std::sync::Arc;

use bevy::{
    math::bounding::Aabb2d,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use itertools::Itertools;

use crate::{
    direction::Direction,
    player::{PlayerChunkMoveEvent, PlayerSpawnEvent},
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
    pub(super) fn spawn_chunks(
        mut spawn_events: EventReader<PlayerSpawnEvent>,
        mut chunk_move_events: EventReader<PlayerChunkMoveEvent>,
        noise: Res<Noise>,
        entities: Res<ChunkEntities>,
        params: Res<WorldgenParams>,
        mut tasks: ResMut<ChunkSpawningTasks>,
    ) {
        fn renderable_chunks(player_offset: IVec2) -> impl Iterator<Item = IVec2> {
            let player_offset = IVec2::new(player_offset.x, player_offset.y);
            let iter_x = (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32)
                .map(move |x| x + player_offset.x);
            let iter_z = (-(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32)
                .map(move |z| z + player_offset.y);

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

        let thread_pool = AsyncComputeTaskPool::get();
        let noise = Arc::new(noise.clone());

        for player_offset in spawn_events
            .read()
            .map(|ev| ev.offset)
            .chain(chunk_move_events.read().map(|ev| ev.new_offset))
        {
            for offset in renderable_chunks(player_offset) {
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
        mut commands: Commands,
        mut chunk_move_events: EventReader<PlayerChunkMoveEvent>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut entities: ResMut<ChunkEntities>,
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
                dirty.0.remove(offset);
                if let Some(entity) = entities.0.remove(offset) {
                    commands.entity(entity).despawn();
                }
                spawning_tasks.0.remove(offset);
                meshing_tasks.0.remove(offset);

                for dir in [
                    Direction::North,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                ] {
                    let neighbor_offset = *offset + IVec2::from(dir);
                    if chunks.0.contains_key(&neighbor_offset) {
                        dirty.0.insert(neighbor_offset);
                    }
                }
            }
        }
    }

    pub(super) fn handle_spawning_tasks(
        mut commands: Commands,
        texture: Res<ChunkTexture>,
        mut tasks: ResMut<ChunkSpawningTasks>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut entities: ResMut<ChunkEntities>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
    ) {
        tasks.0.retain(|&offset, task| {
            if let Some(chunk) = block_on(future::poll_once(task)) {
                let entity = commands
                    .spawn(MaterialMeshBundle {
                        material: materials.add(ChunkMaterial::new(offset, texture.0.clone())),
                        ..Default::default()
                    })
                    .id();

                chunks.0.insert(offset, chunk.into());
                dirty.0.insert(offset);
                entities.0.insert(offset, entity);

                for dir in [
                    Direction::North,
                    Direction::East,
                    Direction::South,
                    Direction::West,
                ] {
                    let neighbor_offset = offset + IVec2::from(dir);
                    if chunks.0.contains_key(&neighbor_offset) {
                        dirty.0.insert(neighbor_offset);
                    }
                }
                false
            } else {
                true
            }
        });
    }
}
