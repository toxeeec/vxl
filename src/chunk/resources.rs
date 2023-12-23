use std::sync::Arc;

use super::{systems::ChunkSpawningTask, Chunk};
use crate::{
    block::{generate_chunk, Blocks, VisibleChunksIterator},
    direction::Direction,
    position::Offset,
    settings::WORLD_WIDTH,
    texture::ChunkTexture,
};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool, utils::HashMap};
use noise::*;

#[derive(Resource, Debug)]
pub(crate) struct Chunks {
    pub(crate) entities: HashMap<Offset, Entity>,
    pub(crate) blocks: Blocks,
}

impl Chunks {
    pub(super) fn for_each_neighbor(&self, offset: Offset, mut f: impl FnMut(Entity)) {
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            if let Some(&e) = self.entities.get(&(offset + Offset::from(dir))) {
                f(e)
            }
        }
    }
}

impl FromWorld for Chunks {
    fn from_world(world: &mut World) -> Self {
        let center_offset = *world.resource::<CenterOffset>();
        let material = world.resource::<ChunkTexture>().material.clone();

        let perlin = Arc::new(Fbm::<Perlin>::new(0).set_frequency(0.005));
        let thread_pool = AsyncComputeTaskPool::get();
        let mut entities = HashMap::with_capacity((WORLD_WIDTH * WORLD_WIDTH) as usize);

        VisibleChunksIterator::new(center_offset).for_each(|offset| {
            let perlin = perlin.clone();
            let task = thread_pool.spawn(async move { (offset, generate_chunk(&perlin, offset)) });
            entities.insert(
                offset,
                world
                    .spawn((
                        MaterialMeshBundle {
                            transform: offset.into(),
                            material: material.clone(),
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        Chunk,
                        ChunkSpawningTask(task),
                    ))
                    .id(),
            );
        });

        Self {
            entities,
            blocks: Blocks::new(perlin),
        }
    }
}

#[derive(Resource, Default, Clone, Copy, Debug)]
pub(crate) struct CenterOffset(pub(crate) Offset);
