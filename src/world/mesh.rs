use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use strum::IntoEnumIterator;

use crate::{direction::Direction, texture::ATTRIBUTE_DATA};

use super::{ChunkEntities, Chunks, DirtyChunks, WorldPlugin, CHUNK_HEIGHT, CHUNK_WIDTH};

const FACE_INDICES: [u32; 6] = [0, 2, 1, 0, 3, 2];

#[derive(Resource, Default, Debug)]
pub(super) struct ChunkMeshingTasks(pub(super) HashMap<IVec2, Task<Mesh>>);

impl WorldPlugin {
    pub(super) fn handle_meshing_tasks(
        mut commands: Commands,
        entities: Res<ChunkEntities>,
        mut tasks: ResMut<ChunkMeshingTasks>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        tasks.0.retain(|offset, task| {
            if let Some(mesh) = block_on(future::poll_once(task)) {
                if let Some(&entity) = entities.0.get(offset) {
                    commands.entity(entity).insert(meshes.add(mesh));
                }
                false
            } else {
                true
            }
        });
    }

    pub(super) fn mesh_chunks(
        chunks: Res<Chunks>,
        mut dirty: ResMut<DirtyChunks>,
        mut tasks: ResMut<ChunkMeshingTasks>,
    ) {
        let thread_pool = AsyncComputeTaskPool::get();

        for &offset in dirty.0.iter() {
            let chunk = chunks.0.get(&offset).unwrap().clone();
            let neighbors = chunks.get_neighbors(offset);

            let task = thread_pool.spawn(async move {
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
                        if chunk
                            .block_at(&neighbors, local_pos + IVec3::from(dir))
                            .is_opaque()
                        {
                            continue;
                        }
                        indices.extend(FACE_INDICES.map(|idx| vertices.len() as u32 + idx));
                        let mut data = block as i32;
                        data = (data << 3) | dir as i32;
                        data =
                            (data << (CHUNK_WIDTH.ilog2() * 2 + CHUNK_HEIGHT.ilog2())) | i as i32;

                        vertices.extend([data; 4]);
                    }
                }

                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(ATTRIBUTE_DATA, vertices)
                .with_inserted_indices(Indices::U32(indices))
            });

            tasks.0.insert(offset, task);
        }

        dirty.0.clear();
    }
}
