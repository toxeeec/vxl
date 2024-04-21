use array_init::array_init;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use strum::IntoEnumIterator;

use crate::direction::Direction;

pub(super) const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 128;
const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

#[derive(Debug)]
struct Chunk([bool; CHUNK_VOLUME]);

#[derive(Debug)]
pub(super) struct WorldPlugin;

impl Chunk {
    fn block_at(&self, pos: IVec3) -> bool {
        if pos.min_element() < 0
            || pos.xz().max_element() >= CHUNK_WIDTH as i32
            || pos.y >= CHUNK_HEIGHT as i32
        {
            return false;
        }

        let i = pos.x + pos.y * (CHUNK_WIDTH * CHUNK_WIDTH) as i32 + pos.z * CHUNK_WIDTH as i32;
        self.0[i as usize]
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self(array_init(|i| {
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            z >= y
        }))
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup);
    }
}

impl WorldPlugin {
    fn setup(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let chunk = Chunk::default();

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for (i, block) in chunk.0.into_iter().enumerate() {
            if !block {
                continue;
            }
            let x = i % CHUNK_WIDTH;
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            let pos = IVec3::new(x as i32, y as i32, z as i32);
            for (vertices, dir) in BLOCK_VERTICES.iter().zip(Direction::iter()) {
                if chunk.block_at(pos + IVec3::from(dir)) {
                    continue;
                }
                indices.extend(FACE_INDICES.map(|idx| positions.len() as u32 + idx));
                positions.extend(vertices.map(|vertex| vertex + pos.as_vec3()));
            }
        }

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_indices(Indices::U32(indices));

        commands.spawn(MaterialMeshBundle {
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                unlit: true,
                ..Default::default()
            }),
            mesh: meshes.add(mesh),

            ..Default::default()
        });
    }
}

#[rustfmt::skip]
const BLOCK_VERTICES: [[Vec3; 4]; 6] = [
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
