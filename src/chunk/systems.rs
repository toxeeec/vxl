use super::{
    bundles::ChunkBundle,
    materials::ChunkMaterial,
    offset::{transform_from_offset, visible_chunks_offsets},
    resources::Chunks,
    Chunk, Dirty,
};
use crate::{
    block::{global_to_local_pos, pos_to_index, Block, BlockBundle, BlockId},
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

const VERTICES_CAPACITY: usize =
    CHUNK_VOLUME as usize / 2 * FACES_VERTICES.len() * FACES_VERTICES[0].len();
const INDICES_CAPACITY: usize =
    CHUNK_VOLUME as usize / 2 * FACES_VERTICES.len() * FACE_INDICES.len();

pub(super) fn spawn_chunks(mut commands: Commands) {
    let mut chunks = Vec::with_capacity((WORLD_WIDTH * WORLD_WIDTH) as usize);
    for offset in visible_chunks_offsets() {
        let transform = transform_from_offset(offset);
        let mut chunk = commands.spawn(ChunkBundle::new(transform));
        let chunk_pos = transform.translation.as_ivec3();
        chunk.with_children(|parent| {
            for i in 0..(CHUNK_VOLUME) {
                parent.spawn(BlockBundle::new(
                    match i / CHUNK_AREA {
                        0..=2 => BlockId::Dirt,
                        3 => BlockId::Grass,
                        _ => BlockId::Air,
                    },
                    IVec3 {
                        x: (i % CHUNK_WIDTH),
                        y: (i / CHUNK_AREA),
                        z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH),
                    } + chunk_pos,
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
    q_block: Query<(&Block, &Visibility)>,
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

        for (&block, visibility) in q_block.iter_many(blocks) {
            if visibility == Visibility::Hidden {
                continue;
            }
            for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                let neighbor_pos = block.pos + IVec3::from(dir);
                if let Some(chunk) = chunks.get_by_pos(neighbor_pos) {
                    let (_, chunk) = q_chunk.get(chunk).unwrap();
                    let local_pos = global_to_local_pos(neighbor_pos);
                    let (_, visibility) = q_block.get(chunk[pos_to_index(local_pos)]).unwrap();
                    if visibility == Visibility::Visible {
                        continue;
                    }
                }
                indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
                uvs.extend(atlas_uvs(atlas, block, dir));
                for vertex in vertices {
                    positions.push(vertex + block.pos.as_vec3());
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
