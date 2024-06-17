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
use strum::IntoEnumIterator;

use crate::{
    block::BlockId,
    direction::Direction,
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

#[derive(Component, Debug)]
struct ChunkSpawningTask(Task<(IVec2, Chunk)>);

#[derive(Component, Debug)]
struct ChunkMeshingTask(Task<Mesh>);

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
            .add_systems(Startup, Self::setup)
            .add_systems(
                Update,
                (
                    Self::handle_meshing_tasks,
                    (
                        Self::handle_spawning_tasks.run_if(resource_exists::<ChunkTexture>),
                        Self::mesh_chunks,
                    )
                        .chain(),
                ),
            );
    }
}

impl WorldPlugin {
    fn setup(mut commands: Commands) {
        let thread_pool = AsyncComputeTaskPool::get();

        for x in -(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32 {
            for z in -(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32 {
                let offset = IVec2::new(x, z);
                let aabb = Aabb2d::new(offset.as_vec2(), Vec2::splat(0.5));
                let distance = Vec2::ZERO.distance(aabb.closest_point(Vec2::ZERO));
                if distance > RENDER_DISTANCE as f32 {
                    continue;
                }
                let task = thread_pool.spawn(async move { (offset, Chunk::default()) });
                commands.spawn(ChunkSpawningTask(task));
            }
        }
    }

    fn handle_spawning_tasks(
        mut commands: Commands,
        mut query: Query<(Entity, &mut ChunkSpawningTask)>,
        texture: Res<ChunkTexture>,
        mut chunks: ResMut<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut entities: ResMut<ChunkEntities>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
    ) {
        for (entity, mut task) in &mut query {
            if let Some((offset, chunk)) = block_on(future::poll_once(&mut task.0)) {
                commands
                    .entity(entity)
                    .remove::<ChunkSpawningTask>()
                    .insert(MaterialMeshBundle {
                        material: materials.add(ChunkMaterial::new(offset, texture.0.clone())),
                        ..Default::default()
                    });

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
            }
        }
    }

    fn handle_meshing_tasks(
        mut commands: Commands,
        mut query: Query<(Entity, &mut ChunkMeshingTask)>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (entity, mut task) in &mut query {
            if let Some(mesh) = block_on(future::poll_once(&mut task.0)) {
                commands
                    .entity(entity)
                    .remove::<ChunkMeshingTask>()
                    .insert(meshes.add(mesh));
            }
        }
    }

    fn mesh_chunks(
        mut commands: Commands,
        chunks: Res<Chunks>,
        entities: Res<ChunkEntities>,
        mut dirty: ResMut<DirtyChunks>,
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

            commands
                .entity(*entities.0.get(&offset).unwrap())
                .insert(ChunkMeshingTask(task));
        }

        dirty.0.clear();
    }
}
