mod db;
mod gen;
mod mesh;
mod spawn;

use std::sync::Arc;

use array_init::array_init;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use db::Db;
use gen::LoadingWorldgenParams;
use mesh::ChunkMeshingTasks;
use spawn::ChunkSpawningTasks;

use crate::{
    block::BlockId, direction::Direction, sets::LoadingSet, state::AppState,
    textures::BlocksTexture,
};

pub(super) use gen::{Noise, WorldgenParams};

pub(super) const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 256;
const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;

#[derive(Debug)]
struct Chunk([BlockId; CHUNK_VOLUME]);

#[derive(Resource, Default, Debug)]
pub(super) struct Chunks(HashMap<IVec2, Arc<Chunk>>);

#[derive(Resource, Default, Debug)]
struct DirtyChunks(HashSet<IVec2>);

#[derive(Resource, Default, Debug)]
struct ChunkEntities(HashMap<IVec2, Entity>);

type Neighbors = [Option<Arc<Chunk>>; 4];

#[derive(Debug)]
pub(super) struct WorldPlugin;

impl Chunk {
    fn block_at(&self, neighbors: &Neighbors, pos: IVec3) -> BlockId {
        debug_assert!(pos.xz().min_element() >= -1 && pos.xz().max_element() <= CHUNK_WIDTH as i32);

        if pos.y < 0 || pos.y >= CHUNK_HEIGHT as i32 {
            return BlockId::Air;
        }

        let offset = pos.xz().div_euclid(IVec2::splat(CHUNK_WIDTH as i32));

        let pos = pos.rem_euclid(IVec3::new(
            CHUNK_WIDTH as i32,
            CHUNK_HEIGHT as i32,
            CHUNK_WIDTH as i32,
        ));

        let chunk = match offset {
            IVec2::ZERO => Some(self),
            _ => neighbors[Direction::try_from(offset).unwrap() as usize]
                .as_ref()
                .map(Arc::as_ref),
        };

        match chunk {
            Some(chunk) => {
                let i =
                    pos.x + pos.y * (CHUNK_WIDTH * CHUNK_WIDTH) as i32 + pos.z * CHUNK_WIDTH as i32;
                chunk.0.get(i as usize).cloned().unwrap_or(BlockId::Air)
            }
            None => BlockId::Air,
        }
    }
}

impl Chunks {
    pub(super) fn block_at(&self, pos: IVec3) -> Option<BlockId> {
        let offset = pos.xz().div_euclid(IVec2::splat(CHUNK_WIDTH as i32));

        self.0.get(&offset).map(|chunk| {
            let local_pos = pos
                - IVec3::new(
                    pos.x.div_euclid(CHUNK_WIDTH as i32),
                    0,
                    pos.z.div_euclid(CHUNK_WIDTH as i32),
                ) * CHUNK_WIDTH as i32;

            chunk.block_at(&self.get_neighbors(offset), local_pos)
        })
    }

    fn get_neighbors(&self, offset: IVec2) -> Neighbors {
        array_init(|i| {
            let dir = [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ][i];
            self.0.get(&(offset + IVec2::from(dir))).cloned()
        })
    }

    pub(super) fn traverse(&self, ray: Ray3d, max: f32) -> Option<(IVec3, BlockId)> {
        fn offset(start: f32, diff: f32) -> f32 {
            if diff.is_sign_positive() {
                start.floor() + 1.0 - start
            } else {
                start - start.floor()
            }
        }

        let start = ray.origin;
        let end = ray.get_point(max);
        let dir = (end - start).normalize();
        let step = dir.signum().as_ivec3();

        let mut pos = start.floor().as_ivec3();

        let mut t_max = Vec3::new(
            offset(start.x, dir.x),
            offset(start.y, dir.y),
            offset(start.z, dir.z),
        ) / dir.abs();
        let t_delta = Vec3::ONE / dir.abs();

        let dist = ((end.x.floor() - start.x.floor()).abs()
            + (end.y.floor() - start.y.floor()).abs()
            + (end.z.floor() - start.z.floor()).abs()) as i32;

        for _ in 0..=dist {
            if let Some(block) = self.block_at(pos).take_if(|block| block.is_solid()) {
                return Some((pos, block));
            }

            if t_max.x < t_max.y && t_max.x < t_max.z {
                pos.x += step.x;
                t_max.x += t_delta.x;
            } else if t_max.y < t_max.z {
                pos.y += step.y;
                t_max.y += t_delta.y;
            } else {
                pos.z += step.z;
                t_max.z += t_delta.z;
            }
        }

        None
    }
}

impl DirtyChunks {
    pub(super) fn insert(&mut self, offset: IVec2) {
        self.0.insert(offset);

        for dir in [
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            self.0.insert(offset + IVec2::from(dir));
        }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>()
            .init_resource::<DirtyChunks>()
            .init_resource::<ChunkEntities>()
            .init_resource::<ChunkSpawningTasks>()
            .init_resource::<ChunkMeshingTasks>()
            .init_resource::<Noise>()
            .init_resource::<LoadingWorldgenParams>()
            .init_resource::<Db>()
            .add_systems(OnEnter(AppState::Generating), Self::generate_world)
            .add_systems(Update, (Self::create_worldgen_params).in_set(LoadingSet))
            .add_systems(
                Update,
                (
                    Self::despawn_chunks,
                    (
                        Self::sync_dirty_chunks,
                        Self::sync_chunk_entities.run_if(resource_exists::<BlocksTexture>),
                    ),
                    (
                        Self::spawn_chunks.run_if(resource_exists::<WorldgenParams>),
                        Self::handle_meshing_tasks,
                        (Self::handle_spawning_tasks, Self::mesh_chunks).chain(),
                    ),
                )
                    .chain(),
            );
    }
}

impl WorldPlugin {
    pub(super) fn is_loaded(params: Option<Res<WorldgenParams>>) -> bool {
        params.is_some()
    }

    pub(super) fn is_generated(chunks: Res<Chunks>) -> bool {
        !chunks.0.is_empty()
    }
}
