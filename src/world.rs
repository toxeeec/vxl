use std::cmp::Ordering;

use array_init::array_init;
use bevy::{
    math::bounding::Aabb2d,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    utils::HashMap,
};
use strum::IntoEnumIterator;

use crate::{
    block::BlockId,
    direction::Direction,
    settings::RENDER_DISTANCE,
    texture::{ChunkMaterial, ChunkTexture, ATTRIBUTE_DATA},
};

pub(super) const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 128;
const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

#[derive(Debug)]
struct Chunk([BlockId; CHUNK_VOLUME]);

#[derive(Resource, Debug)]
struct Chunks(HashMap<IVec2, Chunk>);

#[derive(Debug)]
pub(super) struct WorldPlugin;

impl Chunk {
    fn block_at(&self, pos: IVec3) -> BlockId {
        debug_assert!(
            pos.min_element() >= 0
                && pos.xz().max_element() < CHUNK_WIDTH as i32
                && pos.y < CHUNK_HEIGHT as i32
        );

        let i = pos.x + pos.y * (CHUNK_WIDTH * CHUNK_WIDTH) as i32 + pos.z * CHUNK_WIDTH as i32;
        self.0.get(i as usize).cloned().unwrap_or(BlockId::Air)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self(array_init(|i| {
            let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
            let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

            match z.cmp(&y) {
                Ordering::Less => BlockId::Air,
                Ordering::Equal => BlockId::Grass,
                Ordering::Greater => BlockId::Dirt,
            }
        }))
    }
}

impl Chunks {
    fn block_at(&self, pos: IVec3) -> BlockId {
        match self
            .0
            .get(&pos.xz().div_euclid(IVec2::splat(CHUNK_WIDTH as i32)))
        {
            Some(chunk) => chunk.block_at(
                pos & IVec3::new(
                    CHUNK_WIDTH as i32 - 1,
                    CHUNK_HEIGHT as i32 - 1,
                    CHUNK_WIDTH as i32 - 1,
                ),
            ),
            None => BlockId::Air,
        }
    }
}

impl Default for Chunks {
    fn default() -> Self {
        let mut chunks = HashMap::new();
        for x in -(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32 {
            for z in -(RENDER_DISTANCE as i32)..=RENDER_DISTANCE as i32 {
                let offset = IVec2::new(x, z);
                let aabb = Aabb2d::new(offset.as_vec2(), Vec2::splat(0.5));
                let distance = Vec2::ZERO.distance(aabb.closest_point(Vec2::ZERO));
                if distance <= RENDER_DISTANCE as f32 {
                    chunks.insert(offset, Chunk::default());
                }
            }
        }
        Self(chunks)
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>()
            .add_systems(Update, Self::setup.run_if(resource_added::<ChunkTexture>));
    }
}

impl WorldPlugin {
    fn setup(
        mut commands: Commands,
        chunks: Res<Chunks>,
        texture: Res<ChunkTexture>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ChunkMaterial>>,
    ) {
        for (offset, chunk) in chunks.0.iter() {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            for (i, block) in chunk.0.into_iter().enumerate() {
                if block.is_transparent() {
                    continue;
                }
                let x = i % CHUNK_WIDTH;
                let z = (i / CHUNK_WIDTH) % CHUNK_WIDTH;
                let y = (i / CHUNK_WIDTH / CHUNK_WIDTH) % CHUNK_HEIGHT;

                let local_pos = IVec3::new(x as i32, y as i32, z as i32);

                for dir in Direction::iter() {
                    let global_pos = local_pos
                        + IVec3::new(offset.x, 0, offset.y) * CHUNK_WIDTH as i32
                        + IVec3::from(dir);
                    if chunks.block_at(global_pos).is_opaque() {
                        continue;
                    }
                    indices.extend(FACE_INDICES.map(|idx| vertices.len() as u32 + idx));
                    let mut data = block as i32;
                    data = (data << 3) | dir as i32;
                    data = (data << (CHUNK_WIDTH.ilog2() * 2 + CHUNK_HEIGHT.ilog2())) | i as i32;

                    vertices.extend([data; 4]);
                }
            }

            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(ATTRIBUTE_DATA, vertices)
            .with_inserted_indices(Indices::U32(indices));

            commands.spawn(MaterialMeshBundle {
                material: materials.add(ChunkMaterial::new(*offset, texture.0.clone())),
                mesh: meshes.add(mesh),
                ..Default::default()
            });
        }
    }
}
