use super::{
    resources::{CenterOffset, Chunks},
    Chunk, Dirty,
};
use crate::{
    block::{generate_chunk, BlockId, Transparency, VisibleChunksIterator},
    direction::Direction,
    position::{GlobalPosition, LocalPosition, Offset},
    settings::CHUNK_VOLUME,
    texture::{atlas_uvs, ChunkTexture, ATTRIBUTE_DIRECTION},
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::{block_on, AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use std::sync::Arc;
use strum::IntoEnumIterator;

#[rustfmt::skip]
const FACES_VERTICES: [[Vec3; 4]; 6] = [
    [
        Vec3 {x: 1.0, y: 1.0, z: 0.0}, // north(-z)
        Vec3 {x: 0.0, y: 1.0, z: 0.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0}
    ],
    [
        Vec3 {x: 1.0, y: 1.0, z: 1.0}, // east(+x)
        Vec3 {x: 1.0, y: 1.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 1.0}, // south(+z)
        Vec3 {x: 1.0, y: 1.0, z: 1.0},
        Vec3 {x: 1.0, y: 0.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 0.0}, // west(-x)
        Vec3 {x: 0.0, y: 1.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 1.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0}
    ],
    [
        Vec3 {x: 0.0, y: 1.0, z: 0.0}, // up(+y)
        Vec3 {x: 1.0, y: 1.0, z: 0.0},
        Vec3 {x: 1.0, y: 1.0, z: 1.0},
        Vec3 {x: 0.0, y: 1.0, z: 1.0}
    ],
    [
        Vec3 {x: 0.0, y: 0.0, z: 1.0}, // down(-y)
        Vec3 {x: 1.0, y: 0.0, z: 1.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0},
        Vec3 {x: 0.0, y: 0.0, z: 0.0}
    ]
];

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

const VERTICES_CAPACITY: usize = CHUNK_VOLUME / 2 * FACES_VERTICES.len() * FACES_VERTICES[0].len();
const INDICES_CAPACITY: usize = CHUNK_VOLUME / 2 * FACES_VERTICES.len() * FACE_INDICES.len();
const DIRECTIONS_CAPACITY: usize = CHUNK_VOLUME / 2 * FACES_VERTICES.len();

#[derive(Component, Debug)]
pub(super) struct ChunkMeshingTask(Task<Mesh>);

#[derive(Component, Debug)]
pub(super) struct ChunkSpawningTask(pub(super) Task<(Offset, Arc<[BlockId]>)>);

pub(super) fn mesh_chunks(
    mut commands: Commands,
    mut q_chunk: Query<(Entity, &Transform), (With<Chunk>, With<Dirty>)>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    chunk_texture: Res<ChunkTexture>,
    chunks: Res<Chunks>,
) {
    let atlas = texture_atlases.get(&chunk_texture.atlas).unwrap().clone();
    let thread_pool = AsyncComputeTaskPool::get();

    for (e, &transform) in &mut q_chunk {
        let atlas = atlas.clone();
        let offset = transform.into();

        if let Some(chunk) = chunks.blocks.get_chunk(offset) {
            let neighbors = chunks.blocks.get_neighboring_chunks(offset);
            let task = thread_pool.spawn(async move {
                let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
                let mut uvs = Vec::with_capacity(VERTICES_CAPACITY);
                let mut indices = Vec::with_capacity(INDICES_CAPACITY);
                let mut directions = Vec::with_capacity(DIRECTIONS_CAPACITY);

                for (i, &block_id) in chunk.iter().enumerate() {
                    if block_id.transparency() == Transparency::Invisible {
                        continue;
                    }
                    let local_pos = LocalPosition::from_index(i);
                    let global_pos = GlobalPosition::from_local(local_pos, transform.into());

                    for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                        let neighbor_pos = global_pos + GlobalPosition::from(dir);
                        match neighbors.get_chunk(neighbor_pos.into()) {
                            Some(neighbor_chunk) => {
                                if let Some(neighbor) =
                                    neighbor_chunk.get(LocalPosition::from(neighbor_pos).to_index())
                                {
                                    if neighbor.transparency() == Transparency::Opaque {
                                        continue;
                                    }
                                }
                            }
                            None => continue,
                        }
                        indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
                        uvs.extend(atlas_uvs(&atlas, block_id, dir));
                        directions.extend(FACES_VERTICES[0].map(|_| dir as u32));
                        for vertex in vertices {
                            positions.push(vertex + IVec3::from(local_pos).as_vec3());
                        }
                    }
                }

                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                mesh.insert_attribute(ATTRIBUTE_DIRECTION, directions);
                mesh.set_indices(Some(Indices::U32(indices)));

                mesh
            });

            commands
                .entity(e)
                .insert(ChunkMeshingTask(task))
                .remove::<Dirty>();
        }
    }
}

pub(super) fn despawn_distant_chunks(
    mut commands: Commands,
    query: Query<&Transform, With<Chunk>>,
    center_offset: Res<CenterOffset>,
    mut chunks: ResMut<Chunks>,
) {
    if !center_offset.is_changed() || center_offset.is_added() {
        return;
    }

    for transform in &query {
        let offset = Offset::from(transform);
        if !offset.in_bounds(center_offset.0) {
            if let Some(e) = chunks.entities.remove(&offset) {
                commands.entity(e).despawn();
            }
            chunks.blocks.remove_chunk(offset);
            chunks.for_each_neighbor(offset, |nbor| {
                commands.entity(nbor).insert(Dirty);
            });
        }
    }
}

pub(super) fn spawn_chunks(
    mut commands: Commands,
    center_offset: Res<CenterOffset>,
    chunk_texture: Res<ChunkTexture>,
    mut chunks: ResMut<Chunks>,
) {
    if !center_offset.is_changed() || center_offset.is_added() {
        return;
    }
    let chunks = &mut *chunks;
    let thread_pool = AsyncComputeTaskPool::get();

    VisibleChunksIterator::new(*center_offset).for_each(|offset| {
        if chunks.entities.contains_key(&offset) {
            return;
        }

        let perlin = chunks.blocks.perlin.clone();
        let task = thread_pool.spawn(async move { (offset, generate_chunk(&perlin, offset)) });

        chunks.entities.insert(
            offset,
            commands
                .spawn((
                    MaterialMeshBundle {
                        transform: offset.into(),
                        material: chunk_texture.material.clone(),
                        visibility: Visibility::Hidden,
                        ..Default::default()
                    },
                    Chunk,
                    ChunkSpawningTask(task),
                ))
                .id(),
        );
    });
}

pub(super) fn handle_meshing_tasks(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Handle<Mesh>,
            &mut Visibility,
            &mut ChunkMeshingTask,
        ),
        With<Chunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut handle, mut visibility, mut task) in &mut query {
        if let Some(mesh) = block_on(future::poll_once(&mut task.0)) {
            *visibility = Visibility::Visible;
            *handle = meshes.add(mesh);
            commands.entity(entity).remove::<ChunkMeshingTask>();
        }
    }
}

pub(super) fn handle_spawning_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkSpawningTask)>,
    mut chunks: ResMut<Chunks>,
) {
    for (entity, mut task) in &mut query {
        if let Some((offset, chunk)) = block_on(future::poll_once(&mut task.0)) {
            chunks.blocks.insert_chunk(offset, chunk);
            commands
                .entity(entity)
                .insert(Dirty)
                .remove::<ChunkSpawningTask>();
            chunks.for_each_neighbor(offset, |nbor| {
                commands.entity(nbor).insert(Dirty);
            });
        }
    }
}
