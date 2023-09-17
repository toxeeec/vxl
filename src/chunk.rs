use crate::block::BlockBundle;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const CHUNK_AREA: usize = CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_VOLUME: usize = CHUNK_AREA * CHUNK_HEIGHT;

#[rustfmt::skip]
const FACE_VERTICES: [[Vec3; 4]; 6] = [
    [
        Vec3 {x:  0.5, y:  0.5, z: -0.5}, // north(-z)
        Vec3 {x: -0.5, y:  0.5, z: -0.5},
        Vec3 {x: -0.5, y: -0.5, z: -0.5},
        Vec3 {x:  0.5, y: -0.5, z: -0.5}
    ],
    [
        Vec3 {x:  0.5, y:  0.5, z:  0.5}, // east(+x)
        Vec3 {x:  0.5, y:  0.5, z: -0.5},
        Vec3 {x:  0.5, y: -0.5, z: -0.5},
        Vec3 {x:  0.5, y: -0.5, z:  0.5}
    ],
    [
        Vec3 {x: -0.5, y:  0.5, z:  0.5}, // south(+z)
        Vec3 {x:  0.5, y:  0.5, z:  0.5},
        Vec3 {x:  0.5, y: -0.5, z:  0.5},
        Vec3 {x: -0.5, y: -0.5, z:  0.5}
    ],
    [
        Vec3 {x: -0.5, y:  0.5, z: -0.5}, // west(-x)
        Vec3 {x: -0.5, y:  0.5, z:  0.5},
        Vec3 {x: -0.5, y: -0.5, z:  0.5},
        Vec3 {x: -0.5, y: -0.5, z: -0.5}
    ],
    [
        Vec3 {x: -0.5, y:  0.5, z: -0.5}, // up(+y)
        Vec3 {x:  0.5, y:  0.5, z: -0.5},
        Vec3 {x:  0.5, y:  0.5, z:  0.5},
        Vec3 {x: -0.5, y:  0.5, z:  0.5}
    ],
    [
        Vec3 {x: -0.5, y: -0.5, z:  0.5}, // down(-y)
        Vec3 {x:  0.5, y: -0.5, z:  0.5},
        Vec3 {x:  0.5, y: -0.5, z: -0.5},
        Vec3 {x: -0.5, y: -0.5, z: -0.5}
    ]
];

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

#[derive(Component)]
pub(crate) struct Chunk;

impl Chunk {
    pub(crate) fn spawn(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
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
                chunk.spawn(BlockBundle::new(Vec3 {
                    x: (i % CHUNK_WIDTH) as f32,
                    y: (i / CHUNK_AREA) as f32,
                    z: ((i / CHUNK_WIDTH) % CHUNK_WIDTH) as f32,
                }));
            }
        });
    }

    pub(crate) fn mesh(
        mut q_chunk: Query<(&mut Handle<Mesh>, &Children), (With<Chunk>, Added<Handle<Mesh>>)>,
        q_block: Query<&Transform>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        for (mut mesh_handle, blocks) in q_chunk.iter_mut() {
            let mut positions = Vec::with_capacity(CHUNK_VOLUME);
            let mut indices = Vec::with_capacity(CHUNK_VOLUME * FACE_VERTICES.len() * 6);
            for &block in blocks.iter() {
                if let Ok(transform) = q_block.get(block) {
                    for vertices in FACE_VERTICES {
                        for index in FACE_INDICES {
                            indices.push(positions.len() as u32 + index);
                        }
                        for vertex in vertices {
                            positions.push(vertex + transform.translation);
                        }
                    }
                }
            }
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.set_indices(Some(Indices::U32(indices)));
            *mesh_handle = meshes.add(mesh);
        }
    }
}
