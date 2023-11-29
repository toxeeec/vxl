use std::mem;

use super::{
    chunk_in_bounds,
    resources::{CenterOffset, Chunks},
    Chunk, Dirty,
};
use crate::{
    block::{generate_blocks, Transparency},
    direction::Direction,
    offset::Offset,
    settings::{CHUNK_VOLUME, RENDER_DISTANCE, WORLD_WIDTH},
    texture::{atlas_uvs, ChunkTexture},
};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
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

pub(super) fn mesh_chunks(
    mut commands: Commands,
    mut q_chunk: Query<(Entity, &Transform, &mut Handle<Mesh>), (With<Chunk>, Added<Dirty>)>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    chunk_texture: Res<ChunkTexture>,
    chunks: Res<Chunks>,
    center_offset: Res<CenterOffset>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let atlas = texture_atlases.get(&chunk_texture.atlas).unwrap();
    for (e, &transform, mut m) in &mut q_chunk {
        let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
        let mut uvs = Vec::with_capacity(VERTICES_CAPACITY);
        let mut indices = Vec::with_capacity(INDICES_CAPACITY);

        for &block in chunks.blocks[Offset::from(transform).as_index(center_offset.curr())].iter() {
            if block.transparency == Transparency::Invisible {
                continue;
            }
            for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                let neighbor_pos = block.pos + IVec3::from(dir);
                if let Some(neighbor) = chunks.get_block(neighbor_pos, center_offset.curr()) {
                    if neighbor.transparency == Transparency::Opaque {
                        continue;
                    }
                }
                indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
                uvs.extend(atlas_uvs(atlas, block, dir));
                for vertex in vertices {
                    positions.push(vertex + block.local_pos().as_vec3());
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        *m = meshes.add(mesh);
        commands.entity(e).remove::<Dirty>();
    }
}

pub(super) fn unload_distant_chunks(
    mut commands: Commands,
    query: Query<&Transform, With<Chunk>>,
    center_offset: Res<CenterOffset>,
    mut chunks: ResMut<Chunks>,
) {
    if !center_offset.is_changed() || center_offset.is_added() {
        return;
    }

    for &transform in &query {
        if !chunk_in_bounds(transform, center_offset.curr()) {
            if let Some(e) = chunks.remove(transform.into()) {
                commands.entity(e).despawn();
            }
            chunks.for_each_neighbor(transform.into(), |nbor| {
                commands.entity(nbor).insert(Dirty);
            });
        }
    }
}

pub(super) fn update_offsets(center_offset: Res<CenterOffset>, mut chunks: ResMut<Chunks>) {
    if !center_offset.is_changed() {
        return;
    }

    for (i, offset) in chunks.offsets.iter_mut().enumerate() {
        *offset = Offset::new(
            (i as i32 % WORLD_WIDTH) + center_offset.curr().0.x - RENDER_DISTANCE,
            (i as i32 / WORLD_WIDTH) + center_offset.curr().0.y - RENDER_DISTANCE,
        );
    }
}

pub(super) fn reorder_chunks(
    query: Query<&Transform, With<Chunk>>,
    center_offset: Res<CenterOffset>,
    mut chunks: ResMut<Chunks>,
) {
    if !center_offset.is_changed() || center_offset.is_added() {
        return;
    }

    chunks.reorder(|old, new| {
        for transform in &query {
            let prev_idx = Offset::from(transform).as_index(center_offset.prev());
            let curr_idx = Offset::from(transform).as_index(center_offset.curr());
            mem::swap(&mut old[prev_idx], &mut new[curr_idx]);
        }
    });
}

pub(super) fn spawn_chunks(
    mut commands: Commands,
    chunk_texture: Res<ChunkTexture>,
    center_offset: Res<CenterOffset>,
    mut chunks: ResMut<Chunks>,
) {
    if !center_offset.is_changed() || center_offset.is_added() {
        return;
    }

    let chunks = &mut *chunks;

    for &offset in &chunks.offsets {
        if chunks.entities.contains_key(&offset) {
            continue;
        }
        let blocks = &mut chunks.blocks[offset.as_index(center_offset.curr())];

        let material = chunk_texture.material.clone();

        for (i, block) in generate_blocks(offset).enumerate() {
            blocks[i] = block;
        }

        chunks.for_each_neighbor(offset, |nbor| {
            commands.entity(nbor).insert(Dirty);
        });
        chunks.entities.entry(offset).or_insert_with(|| {
            commands
                .spawn((
                    MaterialMeshBundle {
                        transform: offset.into(),
                        material: material.clone(),
                        ..Default::default()
                    },
                    Chunk,
                    Dirty,
                ))
                .id()
        });
    }
}
