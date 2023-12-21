use super::{
    resources::{CenterOffset, Chunks},
    Chunk, Dirty,
};
use crate::{
    block::Transparency,
    direction::Direction,
    position::{GlobalPosition, LocalPosition, Offset},
    settings::{CHUNK_VOLUME, RENDER_DISTANCE, WORLD_WIDTH},
    texture::{atlas_uvs, ChunkTexture},
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::{block_on, AsyncComputeTaskPool, Task},
};
use futures_lite::future;
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

#[derive(Component, Debug)]
pub(super) struct ChunkMeshingTask(Task<Option<Mesh>>);

pub(super) fn mesh_chunks(
    mut commands: Commands,
    mut q_chunk: Query<(Entity, &Transform), (With<Chunk>, Added<Dirty>)>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    chunk_texture: Res<ChunkTexture>,
    chunks: Res<Chunks>,
) {
    let atlas = texture_atlases.get(&chunk_texture.atlas).unwrap().clone();
    let thread_pool = AsyncComputeTaskPool::get();

    for (e, &transform) in &mut q_chunk {
        let atlas = atlas.clone();
        let blocks = chunks.blocks.clone();

        commands.entity(e).remove::<Dirty>();
        let task = thread_pool.spawn(async move {
            let blocks = blocks.read().await;
            match blocks.get_chunk(transform.into()) {
                Some(chunk) => {
                    let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
                    let mut uvs = Vec::with_capacity(VERTICES_CAPACITY);
                    let mut indices = Vec::with_capacity(INDICES_CAPACITY);

                    for (i, &block_id) in chunk.iter().enumerate() {
                        if block_id.transparency() == Transparency::Invisible {
                            continue;
                        }
                        let local_pos = LocalPosition::from_index(i);
                        let global_pos = GlobalPosition::from_local(local_pos, transform.into());

                        for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                            let neighbor_pos = global_pos + GlobalPosition::from(dir);
                            match blocks.get_chunk(neighbor_pos.into()) {
                                Some(neighbor_chunk) => {
                                    if let Some(neighbor) = neighbor_chunk
                                        .get(LocalPosition::from(neighbor_pos).to_index())
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
                            for vertex in vertices {
                                positions.push(vertex + IVec3::from(local_pos).as_vec3());
                            }
                        }
                    }

                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.set_indices(Some(Indices::U32(indices)));

                    Some(mesh)
                }
                None => None,
            }
        });
        commands.entity(e).insert(ChunkMeshingTask(task));
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
    let chunks = &mut *chunks;
    let mut blocks = chunks.blocks.blocking_write();

    for transform in &query {
        let offset = Offset::from(transform);
        if !offset.in_bounds(center_offset.0) {
            if let Some(e) = chunks.entities.remove(&offset) {
                commands.entity(e).despawn();
            }
            blocks.remove_chunk(offset);
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
    let mut blocks = chunks.blocks.blocking_write();
    let center_offset = center_offset.0;

    for offset in (0..WORLD_WIDTH * WORLD_WIDTH).map(|i| {
        Offset::new(
            (i % WORLD_WIDTH) + center_offset.0.x - RENDER_DISTANCE,
            (i / WORLD_WIDTH) + center_offset.0.y - RENDER_DISTANCE,
        )
    }) {
        if chunks.entities.contains_key(&offset) {
            continue;
        }
        blocks.generate_chunk(offset);

        chunks.for_each_neighbor(offset, |nbor| {
            commands.entity(nbor).insert(Dirty);
        });

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
                    Dirty,
                ))
                .id(),
        );
    }
}

pub(super) fn handle_meshing_tasks(
    mut commands: Commands,
    mut transform_tasks: Query<
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
    for (entity, mut handle, mut visibility, mut task) in &mut transform_tasks {
        if let Some(Some(mesh)) = block_on(future::poll_once(&mut task.0)) {
            *visibility = Visibility::Visible;
            *handle = meshes.add(mesh);
            commands.entity(entity).remove::<ChunkMeshingTask>();
        }
    }
}
