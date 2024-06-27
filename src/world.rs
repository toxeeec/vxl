use std::{cmp::Ordering, sync::Arc};

use array_init::array_init;
use bevy::{
    math::bounding::Aabb2d,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::{HashMap, HashSet},
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    block::BlockId,
    direction::Direction,
    player::{PlayerChunkMoveEvent, PlayerSpawnEvent},
    settings::RENDER_DISTANCE,
    texture::{ChunkMaterial, ChunkTexture, ATTRIBUTE_DATA},
};

pub(super) const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 128;
const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

#[derive(Debug)]
struct Chunk([BlockId; CHUNK_VOLUME]);

#[derive(Resource, Default, Debug)]
struct Chunks(HashMap<IVec2, Arc<Chunk>>);

#[derive(Resource, Default, Debug)]
struct DirtyChunks(HashSet<IVec2>);

#[derive(Resource, Default, Debug)]
struct ChunkEntities(HashMap<IVec2, Entity>);

#[derive(Resource, Default, Debug)]
struct ChunkSpawningTasks(HashMap<IVec2, Task<Chunk>>);

#[derive(Resource, Default, Debug)]
struct ChunkMeshingTasks(HashMap<IVec2, Task<Mesh>>);

#[derive(Debug)]
pub(super) struct WorldPlugin;

type Neighbors = [Option<Arc<Chunk>>; 4];

impl Chunk {
    fn block_at(&self, neighbors: &Neighbors, pos: IVec3) -> BlockId {
        debug_assert!(
            pos.min_element() >= -1
                && pos.xz().max_element() <= CHUNK_WIDTH as i32
                && pos.y <= CHUNK_HEIGHT as i32
        );

        let offset = pos.xz().div_euclid(IVec2::splat(CHUNK_WIDTH as i32));

        let pos = pos.rem_euclid(IVec3::new(
            CHUNK_WIDTH as i32,
            CHUNK_HEIGHT as i32,
            CHUNK_WIDTH as i32,
        ));

        let chunk = match offset {
            IVec2::ZERO => Some(self),
            _ => neighbors[Direction::try_from(offset).unwrap() as usize]
                .as_ref()
                .map(Arc::as_ref),
        };

        match chunk {
            Some(chunk) => {
                let i =
                    pos.x + pos.y * (CHUNK_WIDTH * CHUNK_WIDTH) as i32 + pos.z * CHUNK_WIDTH as i32;
                chunk.0.get(i as usize).cloned().unwrap_or(BlockId::Air)
            }
            None => BlockId::Air,
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self(array_init(|i| {
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            match z.cmp(&y) {
                Ordering::Less => BlockId::Air,
                Ordering::Equal => BlockId::Grass,
                Ordering::Greater => BlockId::Dirt,
            }
        }))
    }
}

impl Chunks {
    fn get_neighbors(&self, offset: IVec2) -> Neighbors {
        array_init(|i| {
            let dir = [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ][i];
            self.0.get(&(offset + IVec2::from(dir))).cloned()
        })
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>()
            .init_resource::<DirtyChunks>()
            .init_resource::<ChunkEntities>()
            .init_resource::<ChunkSpawningTasks>()
            .init_resource::<ChunkMeshingTasks>()
            .add_systems(
                Update,
                (
                    Self::despawn_chunks,
                    (
                        Self::spawn_chunks,
                        Self::handle_meshing_tasks,
                        (
                            Self::handle_spawning_tasks.run_if(resource_exists::<ChunkTexture>),
                            Self::mesh_chunks,
                        )
                            .chain(),
                    ),
                )
                    .chain(),
            );
    }
}

impl WorldPlugin {
    fn spawn_chunks(
        mut spawn_events: EventReader<PlayerSpawnEvent>,
        mut chunk_move_events: EventReader<PlayerChunkMoveEvent>,
        entities: Res<ChunkEntities>,
        mut tasks: ResMut<ChunkSpawningTasks>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();

        for player_offset in spawn_events
            .read()
            .map(|ev| ev.offset)
            .chain(chunk_move_events.read().map(|ev| ev.new_offset))
        {
            for offset in renderable_chunks(player_offset) {
                if entities.0.contains_key(&offset) {
                    continue;
                }
                let task = thread_pool.spawn(async move { Chunk::default() });
                tasks.0.insert(offset, task);
            }
        }
    }

    fn despawn_chunks(
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

    fn handle_spawning_tasks(
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

    fn handle_meshing_tasks(
        mut commands: Commands,
        entities: Res<ChunkEntities>,
        mut tasks: ResMut<ChunkMeshingTasks>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        tasks.0.retain(|offset, task| {
            if let Some(mesh) = block_on(future::poll_once(task)) {
                if let Some(&entity) = entities.0.get(offset) {
                    commands.entity(entity).insert(meshes.add(mesh));
                }
                false
            } else {
                true
            }
        });
    }

    fn mesh_chunks(
        chunks: Res<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut tasks: ResMut<ChunkMeshingTasks>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();

        for &offset in dirty.0.iter() {
            let chunk = chunks.0.get(&offset).unwrap().clone();
            let neighbors = chunks.get_neighbors(offset);

            let task = thread_pool.spawn(async move {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();

                for (i, block) in chunk.0.into_iter().enumerate() {
                    if block.is_transparent() {
                        continue;
                    }
                    let x = i % CHUNK_WIDTH;
                    let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
                    let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

                    let local_pos = IVec3::new(x as i32, y as i32, z as i32);

                    for dir in Direction::iter() {
                        if chunk
                            .block_at(&neighbors, local_pos + IVec3::from(dir))
                            .is_opaque()
                        {
                            continue;
                        }
                        indices.extend(FACE_INDICES.map(|idx| vertices.len() as u32 + idx));
                        let mut data = block as i32;
                        data = (data << 3) | dir as i32;
                        data =
                            (data << (CHUNK_WIDTH.ilog2() * 2 + CHUNK_HEIGHT.ilog2())) | i as i32;

                        vertices.extend([data; 4]);
                    }
                }

                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(ATTRIBUTE_DATA, vertices)
                .with_inserted_indices(Indices::U32(indices))
            });

            tasks.0.insert(offset, task);
        }

        dirty.0.clear();
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
