use array_init::array_init;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use strum::IntoEnumIterator;

use crate::{direction::Direction, material::ChunkMaterial};

pub(super) const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 128;
const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

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
        mut materials: ResMut<Assets<ChunkMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let chunk = Chunk::default();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for (i, block) in chunk.0.into_iter().enumerate() {
            if !block {
                continue;
            }
            let x = i % CHUNK_WIDTH;
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            let pos = IVec3::new(x as i32, y as i32, z as i32);
            for dir in Direction::iter() {
                if chunk.block_at(pos + IVec3::from(dir)) {
                    continue;
                }
                indices.extend(FACE_INDICES.map(|idx| vertices.len() as u32 + idx));
                let data =
                    (dir as u32) << (CHUNK_WIDTH.ilog2() * 2 + CHUNK_HEIGHT.ilog2()) | i as u32;
                vertices.extend([data; 4]);
            }
        }

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(ChunkMaterial::ATTRIBUTE_DATA, vertices)
        .with_inserted_indices(Indices::U32(indices));

        commands.spawn(MaterialMeshBundle {
            material: materials.add(ChunkMaterial {}),
            mesh: meshes.add(mesh),

            ..Default::default()
        });
    }
}
