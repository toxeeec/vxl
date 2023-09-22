use super::{components::Chunk, CHUNK_AREA, CHUNK_VOLUME, CHUNK_WIDTH, direction::Direction};
use crate::block::{block_visible, BlockBundle};
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

pub(super) fn spawn_chunk(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let pbr = PbrBundle {
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            unlit: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    commands.spawn((pbr, Chunk)).with_children(|chunk| {
        for i in 0..CHUNK_VOLUME {
            chunk.spawn(BlockBundle::new(
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
    mut q_chunk: Query<(&mut Handle<Mesh>, &Children), (With<Chunk>, Added<Handle<Mesh>>)>,
    q_block: Query<(&Transform, &Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mut mesh_handle, blocks) in q_chunk.iter_mut() {
        let mut positions = Vec::with_capacity(VERTICES_CAPACITY);
        let mut indices = Vec::with_capacity(INDICES_CAPACITY);
        mesh_chunk(blocks, &q_block, &mut positions, &mut indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_indices(Some(Indices::U32(indices)));
        *mesh_handle = meshes.add(mesh);
    }
}

fn mesh_chunk(
    blocks: &Children,
    q_block: &Query<(&Transform, &Visibility)>,
    positions: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
) {
    for &block in blocks.iter() {
        if let Ok((&Transform { translation, .. }, visibility)) = q_block.get(block) {
            if visibility == Visibility::Hidden {
                continue;
            }
            for (vertices, dir) in FACES_VERTICES.into_iter().zip(Direction::iter()) {
                if block_visible(translation.as_ivec3() + IVec3::from(dir), blocks, q_block) {
                    continue;
                }
                for index in FACE_INDICES {
                    indices.push(positions.len() as u32 + index);
                }
                for vertex in vertices {
                    positions.push(vertex + translation);
                }
            }
        }
    }
}
