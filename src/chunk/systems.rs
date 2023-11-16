use super::{
    bundles::ChunkBundle, materials::ChunkMaterial, offset::visible_chunks_offsets,
    resources::Chunks, Chunk, Dirty,
};
use crate::{
    block::{global_to_local_pos, pos_to_index, Block, BlockBundle},
    direction::Direction,
    settings::{CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH, WORLD_WIDTH},
    texture::{atlas_uvs, Textures},
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

pub(super) fn spawn_chunks(mut commands: Commands) {
    let mut chunks = Vec::with_capacity(WORLD_WIDTH * WORLD_WIDTH);
    for offset in visible_chunks_offsets() {
        let mut chunk = commands.spawn(ChunkBundle::new(offset));
        chunk.with_children(|parent| {
            for i in 0..CHUNK_VOLUME {
                parent.spawn(BlockBundle::new(
                    match i / CHUNK_AREA {
                        0..=2 => Block::Dirt,
                        3 => Block::Grass,
                        _ => Block::Air,
                    },
                    Vec3 {
                        x: (i % CHUNK_WIDTH) as f32,
                        y: (i / CHUNK_AREA) as f32,
                        z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH) as f32,
                    },
                ));
            }
        });
        chunks.push(chunk.id());
    }
    commands.insert_resource(Chunks::new(chunks));
}

pub(super) fn mesh_chunks(
    mut commands: Commands,
    q_chunk: Query<(Entity, &Children), (With<Chunk>, With<Dirty>)>,
    q_block: Query<(&Block, &GlobalTransform, &Visibility)>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    textures: Res<Textures>,
    chunks: Res<Chunks>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let atlas = texture_atlases.get(&textures.blocks).unwrap();
    for (e, blocks) in q_chunk.iter() {
        let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
        let mut uvs = Vec::with_capacity(VERTICES_CAPACITY);
        let mut indices = Vec::with_capacity(INDICES_CAPACITY);

        for (&block, transform, visibility) in q_block.iter_many(blocks) {
            if visibility == Visibility::Hidden {
                continue;
            }
            for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                let translation = transform.translation();
                let neighbor_pos = translation.as_ivec3() + IVec3::from(dir);
                if let Some(chunk) = chunks.get_by_pos(neighbor_pos) {
                    let (_, chunk) = q_chunk.get(chunk).unwrap();
                    let local_pos = global_to_local_pos(neighbor_pos);
                    let (_, _, visibility) = q_block.get(chunk[pos_to_index(local_pos)]).unwrap();
                    if visibility == Visibility::Visible {
                        continue;
                    }
                }
                indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
                uvs.extend(atlas_uvs(atlas, block, dir));
                for vertex in vertices {
                    positions.push(vertex + translation);
                }
            }
        }

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
        commands.entity(e).remove::<Dirty>();
    }
}
