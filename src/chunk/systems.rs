use super::{components::Chunk, materials::ChunkMaterial, CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH};
use crate::{
    block::{block_visible, Block, BlockBundle},
    direction::Direction,
    texture::{uv::atlas_uvs, Textures},
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

#[derive(Debug)]
struct ChunkMesh {
    positions: Vec<Vec3>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

pub(super) fn spawn_chunk(mut commands: Commands) {
    commands.spawn(Chunk).with_children(|chunk| {
        for i in 0..CHUNK_VOLUME {
            chunk.spawn(BlockBundle::new(
                if i < CHUNK_AREA * 3 {
                    Block::Dirt
                } else {
                    Block::Grass
                },
                Vec3 {
                    x: (i % CHUNK_WIDTH) as f32,
                    y: (i / CHUNK_AREA) as f32,
                    z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH) as f32,
                },
                i < CHUNK_AREA * 4,
            ));
        }
    });
}

pub(super) fn mesh_chunks(
    mut commands: Commands,
    q_chunk: Query<&Children, (With<Chunk>, Added<Chunk>)>,
    q_block: Query<(&Block, &Transform, &Visibility)>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    textures: Res<Textures>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let atlas = texture_atlases.get(&textures.blocks).unwrap();
    for blocks in q_chunk.iter() {
        let ChunkMesh {
            positions,
            uvs,
            indices,
        } = mesh_chunk(blocks, &q_block, atlas);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        commands.spawn(MaterialMeshBundle {
            material: materials.add(ChunkMaterial {
                texture: atlas.texture.clone(),
            }),
            mesh: meshes.add(mesh),
            ..Default::default()
        });
    }
}

fn mesh_chunk(
    blocks: &Children,
    q_block: &Query<(&Block, &Transform, &Visibility)>,
    atlas: &TextureAtlas,
) -> ChunkMesh {
    let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
    let mut uvs = Vec::with_capacity(VERTICES_CAPACITY);
    let mut indices = Vec::with_capacity(INDICES_CAPACITY);
    for &block in blocks.iter() {
        let (&block, &Transform { translation, .. }, visibility) = q_block.get(block).unwrap();
        if visibility == Visibility::Hidden {
            continue;
        }
        for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
            if block_visible(translation.as_ivec3() + IVec3::from(dir), blocks, q_block) {
                continue;
            }
            indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
            uvs.extend(atlas_uvs(atlas, block, dir));
            for vertex in vertices {
                positions.push(vertex + translation);
            }
        }
    }

    ChunkMesh {
        positions,
        uvs,
        indices,
    }
}
